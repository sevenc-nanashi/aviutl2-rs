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

        ::aviutl2::internal_module! {
            impl ::aviutl2::module::ScriptModuleFunctions for #impl_token {
                fn functions() -> Vec<::aviutl2::module::ModuleFunction> {
                    let mut functions = Vec::new();
                    #(#function_tables)*
                    functions
                }
            }

            impl #impl_token {
                #(#function_impls)*
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
            let internal_method_name = syn::Ident::new(
                &format!("__aviutl2_internal_module_function_{}", method_name_str),
                method_name.span(),
            );
            let func_table = quote::quote! {
                functions.push(::aviutl2::module::ModuleFunction {
                    name: #method_name_str.to_string(),
                    func: <#impl_token>::#internal_method_name,
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
                if has_self {
                    quote::quote! {
                        extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                            let params = ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp);
                            let __internal_self = <#impl_token as ::aviutl2::module::ScriptModuleSingleton>::__get_singleton_state();
                            let __internal_self = __internal_self
                                .read()
                                .expect("Plugin handle is not initialized");
                            let __internal_self = &__internal_self
                                .as_ref()
                                .expect("Plugin instance is not initialized")
                                .instance;
                            let () = <#impl_token>::#method_name(__internal_self, &params);
                        }
                    }
                } else {
                    quote::quote! {
                        extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                            let params = ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp);
                            let () = <#impl_token>::#method_name(&params);
                        }
                    }
                }
            } else {
                let params = &method.sig.inputs;
                // Separate receiver and non-receiver parameters
                let mut param_bridges = Vec::new();
                let mut param_index: usize = 0;
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
                            if r.mutability.is_some() {
                                return Err(syn::Error::new_spanned(
                                    r,
                                    "method receiver must be an immutable reference",
                                )
                                .to_compile_error());
                            }

                            param_bridges.push(quote::quote! {
                                let __internal_self = <#impl_token as ::aviutl2::module::__bridge::ScriptModuleSingleton>::__get_singleton_state();
                                let __internal_self = __internal_self
                                    .read()
                                    .expect("Plugin handle is not initialized");
                                let __internal_self = &__internal_self
                                    .as_ref()
                                    .expect("Plugin instance is not initialized")
                                    .instance;
                            });
                        }
                        syn::FnArg::Typed(pat_type) => {
                            let ty = &pat_type.ty;
                            let pat = &pat_type.pat;
                            let idx = param_index;
                            param_bridges.push(quote::quote! {
                                let #pat: #ty = match <#ty as ::aviutl2::module::__bridge::FromScriptModuleParam>::from_param(&params, #idx) {
                                    ::std::option::Option::Some(value) => value,
                                    ::std::option::Option::None => {
                                        params.set_error(&format!(
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
                let param_names = params.iter().map(|param| match param {
                    syn::FnArg::Receiver(_) => quote::quote! { __internal_self },
                    syn::FnArg::Typed(pat_type) => {
                        let pat = &pat_type.pat;
                        quote::quote! { #pat }
                    }
                });
                quote::quote! {
                    extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                        let params = ::aviutl2::module::ScriptModuleCallHandle::from_ptr(smp);
                        #(#param_bridges)*
                        let result = <#impl_token>::#method_name(#(#param_names),*);
                        ::aviutl2::module::ToScriptModuleReturnValue::push_value(&result, &params);
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
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
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
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
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
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
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
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }
}
