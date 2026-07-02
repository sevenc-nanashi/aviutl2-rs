use quote::ToTokens;

use crate::script_module_bridge::{
    ReceiverKind, create_method_bridge, parse_inherent_impl, parse_unwind_attr, wrap_with_unwind,
};

pub fn module_metatable(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let unwind = parse_unwind_attr(attr)?;
    let mut item = parse_inherent_impl(item, "module_metatable")?;
    let impl_token = item.self_ty.to_token_stream();

    let bridges = item
        .items
        .iter_mut()
        .map(|item| create_bridge(&impl_token, item, unwind))
        .collect::<Result<Vec<_>, _>>()?;
    let (method_tables, method_impls): (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ) = bridges.into_iter().unzip();
    let default_gc_table = quote::quote! {
        ::aviutl2::sys::module2::META_METHOD_FUNCTION {
            method: concat!("__gc", "\0").as_ptr() as *const ::std::os::raw::c_char,
            func: __meta_table_gc_method,
        },
    };
    let default_gc_impl = quote::quote! {
        unsafe extern "C" fn __meta_table_gc_method(
            smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM,
        ) {
            unsafe {
                let _ = ::std::sync::Arc::from_raw(
                    (*smp).userdata as *const ::std::sync::Mutex<#impl_token>,
                );
            }
        }
    };

    Ok(quote::quote! {
        #item

        ::aviutl2::__internal_module! {
            impl ::aviutl2::module::AsScriptModuleUserData for #impl_token {
                const META_METHOD_FUNCTIONS: &'static [::aviutl2::sys::module2::META_METHOD_FUNCTION] = &[
                    #(#method_tables)*
                    #default_gc_table
                    ::aviutl2::sys::module2::META_METHOD_FUNCTION {
                        method: ::std::ptr::null(),
                        func: __meta_table_dummy_method,
                    },
                ];
            }

            unsafe extern "C" fn __meta_table_dummy_method(
                _smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM,
            ) {
            }

            #(#method_impls)*
            #default_gc_impl
        }
    })
}

fn create_bridge(
    impl_token: &proc_macro2::TokenStream,
    item: &mut syn::ImplItem,
    unwind: bool,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), proc_macro2::TokenStream> {
    match item {
        syn::ImplItem::Fn(method) => {
            let method_name = method.sig.ident.to_string();
            if method_name == "__gc" {
                return Err(syn::Error::new_spanned(
                    method,
                    "The method name `__gc` is reserved for the garbage collection method. Please impl Drop for your type instead of using `__gc` method.",
                )
                .to_compile_error());
            }
            let bridge = create_method_bridge(impl_token, method, ReceiverKind::UserData)?;
            let internal_method_name = &bridge.internal_method_name;
            let method_name_str = &bridge.method_name_str;
            let method_table = quote::quote! {
                ::aviutl2::sys::module2::META_METHOD_FUNCTION {
                    method: concat!(#method_name, "\0").as_ptr() as *const ::std::os::raw::c_char,
                    func: #internal_method_name,
                },
            };
            let method_impl = wrap_with_unwind(
                internal_method_name,
                method_name_str,
                &bridge.body,
                true,
                unwind,
            );

            Ok((method_table, method_impl))
        }
        _ => Err(syn::Error::new_spanned(
            item,
            "`module_metatable` macro can only be applied to methods",
        )
        .to_compile_error()),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_metatable() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl UserData {
                fn index(&self, key: String) -> i32 {
                    self.get(key)
                }

                fn newindex(&mut self, key: String, value: i32) {
                    self.set(key, value);
                }
            }
        };
        let output = module_metatable(proc_macro2::TokenStream::new(), input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn test_direct_gc() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl UserData {
                #[direct]
                fn gc(&mut self, handle: &mut ::aviutl2::module::ScriptModuleCallHandle) {
                    let _ = handle;
                    self.close();
                }
            }
        };
        let output = module_metatable(proc_macro2::TokenStream::new(), input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn test_direct_no_self() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl UserData {
                #[direct]
                fn call(handle: &mut ::aviutl2::module::ScriptModuleCallHandle) {
                    let _ = handle;
                }
            }
        };
        let output = module_metatable(proc_macro2::TokenStream::new(), input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    fn format_tokens(tokens: proc_macro2::TokenStream) -> String {
        let replaced = tokens
            .to_string()
            .replace(":: aviutl2 :: __internal_module !", "mod __internal_module");
        let replaced = proc_macro2::TokenStream::from_str(&replaced).unwrap();
        let formatted = rustfmt_wrapper::rustfmt(replaced).unwrap();
        formatted.replace("mod __internal_module", "::aviutl2::__internal_module!")
    }
}
