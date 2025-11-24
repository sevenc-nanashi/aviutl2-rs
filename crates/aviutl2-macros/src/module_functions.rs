use quote::ToTokens;

pub fn module_functions(
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut item: syn::ItemImpl = syn::parse2(item).map_err(|e| e.to_compile_error())?;
    if item.trait_.is_some() {
        return Err(syn::Error::new_spanned(
            item,
            "`module_functions` macro can only be applied to inherent impl blocks",
        )
        .to_compile_error());
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            item,
            "`module_functions` macro does not support generic impl blocks",
        )
        .to_compile_error());
    }
    if item.self_ty.to_token_stream().to_string().contains('<') {
        return Err(syn::Error::new_spanned(
            item,
            "`module_functions` macro does not support generic types",
        )
        .to_compile_error());
    }
    let impl_token = item.self_ty.to_token_stream();

    let (function_tables, function_impls): (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ) = item
        .items
        .iter_mut()
        .map(|item| create_bridge(&impl_token, item))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();

    Ok(quote::quote! {
        #item

        ::aviutl2::__internal_module! {
            impl ::aviutl2::module::ScriptModuleFunctions for #impl_token {
                fn functions() -> Vec<::aviutl2::module::ModuleFunction> {
                    let mut functions = Vec::new();
                    #(#function_tables)*
                    return functions;

                    #(#function_impls)*
                }
            }
        }
    })
}

fn create_bridge(
    impl_token: &proc_macro2::TokenStream,
    item: &mut syn::ImplItem,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), proc_macro2::TokenStream> {
    match item {
        syn::ImplItem::Fn(method) => {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();
            let internal_method_name =
                syn::Ident::new(&format!("bridge_{}", method_name), method_name.span());
            let func_table = quote::quote! {
                functions.push(::aviutl2::module::ModuleFunction {
                    name: #method_name_str.to_string(),
                    func: #internal_method_name,
                });
            };

            let direct_index = method
                .attrs
                .iter()
                .position(|attr| attr.path().is_ident("direct"));
            let func_impl = if let Some(direct_index) = direct_index {
                method.attrs.remove(direct_index);
                let has_self = method
                    .sig
                    .inputs
                    .iter()
                    .any(|param| matches!(param, syn::FnArg::Receiver(_)));
                // detect if receiver is &mut self
                let mut_self = method
                    .sig
                    .inputs
                    .iter()
                    .find_map(|param| match param {
                        syn::FnArg::Receiver(r) => Some(r.mutability.is_some()),
                        _ => None,
                    })
                    .unwrap_or(false);
                if has_self {
                    if mut_self {
                        quote::quote! {
                            extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                                let mut params = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp) };
                                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance_mut(|__internal_self| {
                                    let () = <#impl_token>::#method_name(__internal_self, &mut params);
                                });
                            }
                        }
                    } else {
                        quote::quote! {
                            extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                                let mut params = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp) };
                                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance(|__internal_self| {
                                    let () = <#impl_token>::#method_name(__internal_self, &mut params);
                                });
                            }
                        }
                    }
                } else {
                    quote::quote! {
                        extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                            let mut params = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp) };
                            let () = <#impl_token>::#method_name(&mut params);
                        }
                    }
                }
            } else {
                let params = &method.sig.inputs;
                // Separate receiver and non-receiver parameters
                let mut param_bridges = Vec::new();
                let mut param_index: usize = 0;
                let mut has_self = false;
                let mut self_is_mut = false;
                for param in params.iter() {
                    match param {
                        syn::FnArg::Receiver(r) => {
                            if r.reference.is_none() {
                                return Err(syn::Error::new_spanned(
                                    r,
                                    "method receiver must be a reference",
                                )
                                .to_compile_error());
                            }
                            has_self = true;
                            self_is_mut = r.mutability.is_some();
                        }
                        syn::FnArg::Typed(pat_type) => {
                            let ty = &pat_type.ty;
                            let pat = &pat_type.pat;
                            let idx = param_index;
                            param_bridges.push(quote::quote! {
                                let #pat: #ty = match <#ty as ::aviutl2::module::FromScriptModuleParam>::from_param(&params, #idx) {
                                    ::std::option::Option::Some(value) => value,
                                    ::std::option::Option::None => {
                                        let _ = params.set_error(&format!(
                                            "Failed to convert parameter #{} to {}", #idx, stringify!(#ty)
                                        ));
                                        return;
                                    }
                                };
                            });
                            param_index += 1;
                        }
                    }
                }
                let param_names_vec: Vec<proc_macro2::TokenStream> = params
                    .iter()
                    .map(|param| match param {
                        syn::FnArg::Receiver(_) => quote::quote! { __internal_self },
                        syn::FnArg::Typed(pat_type) => {
                            let pat = &pat_type.pat;
                            quote::quote! { #pat }
                        }
                    })
                    .collect();
                if has_self {
                    if self_is_mut {
                        quote::quote! {
                            extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                                let mut params = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp) };
                                #(#param_bridges)*
                                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance_mut(|__internal_self| {
                                    let fn_result = <#impl_token>::#method_name(#(#param_names_vec),*);
                                    let push_result = ::aviutl2::module::IntoScriptModuleReturnValue::push_into(fn_result, &mut params);
                                    let _ = ::aviutl2::module::IntoScriptModuleReturnValue::push_into(push_result, &mut params);
                                });
                            }
                        }
                    } else {
                        quote::quote! {
                            extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                                let mut params = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp) };
                                #(#param_bridges)*
                                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance(|__internal_self| {
                                    let fn_result = <#impl_token>::#method_name(#(#param_names_vec),*);
                                    let push_result = ::aviutl2::module::IntoScriptModuleReturnValue::push_into(fn_result, &mut params);
                                    let _ = ::aviutl2::module::IntoScriptModuleReturnValue::push_into(push_result, &mut params);
                                });
                            }
                        }
                    }
                } else {
                    quote::quote! {
                        extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                            let mut params = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp) };
                            #(#param_bridges)*
                            let fn_result = <#impl_token>::#method_name(#(#param_names_vec),*);
                            let push_result = ::aviutl2::module::IntoScriptModuleReturnValue::push_into(fn_result, &mut params);
                            let _ = ::aviutl2::module::IntoScriptModuleReturnValue::push_into(push_result, &mut params);
                        }
                    }
                }
            };

            Ok((func_table, func_impl))
        }
        _ => Err(syn::Error::new_spanned(
            item,
            "`module_functions` macro can only be applied to methods",
        )
        .to_compile_error()),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_no_self() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl MyModule {
                fn my_function(hoge: i32) -> i32 {
                    hoge + 1
                }
            }
        };
        let output = module_functions(input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn test_with_self() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl MyModule {
                fn my_function(&self, fuga: f64) -> f64 {
                    fuga * 2.0
                }
            }
        };
        let output = module_functions(input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn test_direct() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl MyModule {
                #[direct]
                fn my_function(&self) {
                    // do something
                }
            }
        };
        let output = module_functions(input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn test_direct_no_self() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl MyModule {
                #[direct]
                fn my_function() {
                    // do something
                }
            }
        };
        let output = module_functions(input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    fn format_tokens(tokens: proc_macro2::TokenStream) -> String {
        // マクロだと rustfmt がうまく動かないので、フォーマットできるように置換する
        let replaced = tokens
            .to_string()
            .replace(":: aviutl2 :: __internal_module !", "mod __internal_module");
        let replaced = proc_macro2::TokenStream::from_str(&replaced).unwrap();
        let formatted = rustfmt_wrapper::rustfmt(replaced).unwrap();
        // 元に戻す
        formatted.replace("mod __internal_module", "::aviutl2::__internal_module!")
    }
}
