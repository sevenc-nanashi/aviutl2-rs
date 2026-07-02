use quote::ToTokens;

use crate::script_module_bridge::{
    ReceiverKind, create_method_bridge, parse_inherent_impl, parse_unwind_attr, wrap_with_unwind,
};

pub fn module_functions(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let unwind = parse_unwind_attr(attr)?;
    let mut item = parse_inherent_impl(item, "module_functions")?;
    let impl_token = item.self_ty.to_token_stream();

    let (function_tables, function_impls): (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ) = item
        .items
        .iter_mut()
        .map(|item| create_bridge(&impl_token, item, unwind))
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
    unwind: bool,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), proc_macro2::TokenStream> {
    match item {
        syn::ImplItem::Fn(method) => {
            let bridge =
                create_method_bridge(impl_token, method, ReceiverKind::ScriptModuleSingleton)?;
            let method_name_str = &bridge.method_name_str;
            let internal_method_name = &bridge.internal_method_name;
            let func_table = quote::quote! {
                functions.push(::aviutl2::module::ModuleFunction {
                    name: #method_name_str.to_string(),
                    func: #internal_method_name,
                });
            };
            let func_impl = wrap_with_unwind(
                internal_method_name,
                method_name_str,
                &bridge.body,
                false,
                unwind,
            );

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
        let output = module_functions(proc_macro2::TokenStream::new(), input).unwrap();
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
        let output = module_functions(proc_macro2::TokenStream::new(), input).unwrap();
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
        let output = module_functions(proc_macro2::TokenStream::new(), input).unwrap();
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
        let output = module_functions(proc_macro2::TokenStream::new(), input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn test_unwind_meta() {
        let input: proc_macro2::TokenStream = quote::quote! {
            impl MyModule {
                fn my_function(hoge: i32) -> i32 {
                    hoge + 1
                }
            }
        };
        let attr = quote::quote! { unwind = true };
        let output = module_functions(attr, input).unwrap();
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
