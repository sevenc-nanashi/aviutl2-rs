pub fn script_module_callback(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let expr: syn::ExprClosure = syn::parse2(input).map_err(|e| e.to_compile_error())?;
    let (param_bridges, param_names) = parse_params(&expr)?;

    Ok(quote::quote! {
        {
            let mut __callback = #expr;
            let __callback: ::std::boxed::Box<
                dyn FnMut(&mut ::aviutl2::module::ScriptModuleCallHandle) + Send
            > = ::std::boxed::Box::new(move |__handle| {
                #(#param_bridges)*
                let fn_result = __callback(#(#param_names),*);
                ::aviutl2::module::__push_return_value(__handle, fn_result);
            });
            let __callback = ::std::sync::Mutex::new(__callback);
            let __callback = ::std::boxed::Box::new(__callback);
            let __userdata = ::std::boxed::Box::into_raw(__callback) as *mut ::std::ffi::c_void;
            unsafe extern "C" fn __script_module_callback_trampoline(
                smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM,
            ) {
                let __callback = unsafe {
                    &*((*smp).userdata as *const ::std::sync::Mutex<
                        ::std::boxed::Box<
                            dyn FnMut(&mut ::aviutl2::module::ScriptModuleCallHandle) + Send
                        >
                    >)
                };
                let mut __handle = unsafe {
                    ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp)
                };
                if let Err(panic_info) = ::aviutl2::__catch_unwind_with_panic_info(
                    ::std::panic::AssertUnwindSafe(|| {
                        let mut __callback = __callback.lock().expect("script module callback mutex poisoned");
                        (__callback)(&mut __handle);
                    }),
                ) {
                    ::aviutl2::tracing::error!(
                        "Panic occurred during script module callback: {}",
                        panic_info
                    );
                    let _ = ::aviutl2::logger::write_error_log(&panic_info);
                }
            }
            ::aviutl2::module::ScriptModuleFunctionCallback {
                func: __script_module_callback_trampoline,
                userdata: __userdata,
            }
        }
    })
}

pub fn script_module_direct_callback(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let expr: syn::ExprClosure = syn::parse2(input).map_err(|e| e.to_compile_error())?;
    if expr.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            expr,
            "`script_module_direct_callback` expects exactly one `ScriptModuleCallHandle` parameter",
        )
        .to_compile_error());
    }

    Ok(quote::quote! {
        {
            let __callback: ::std::boxed::Box<
                dyn FnMut(&mut ::aviutl2::module::ScriptModuleCallHandle) + Send
            > = ::std::boxed::Box::new(#expr);
            let __callback = ::std::sync::Mutex::new(__callback);
            let __callback = ::std::boxed::Box::new(__callback);
            let __userdata = ::std::boxed::Box::into_raw(__callback) as *mut ::std::ffi::c_void;
            unsafe extern "C" fn __script_module_callback_trampoline(
                smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM,
            ) {
                let __callback = unsafe {
                    &*((*smp).userdata as *const ::std::sync::Mutex<
                        ::std::boxed::Box<
                            dyn FnMut(&mut ::aviutl2::module::ScriptModuleCallHandle) + Send
                        >
                    >)
                };
                let mut __handle = unsafe {
                    ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp)
                };
                if let Err(panic_info) = ::aviutl2::__catch_unwind_with_panic_info(
                    ::std::panic::AssertUnwindSafe(|| {
                        let mut __callback = __callback.lock().expect("script module callback mutex poisoned");
                        let () = (__callback)(&mut __handle);
                    }),
                ) {
                    ::aviutl2::tracing::error!(
                        "Panic occurred during script module callback: {}",
                        panic_info
                    );
                    let _ = ::aviutl2::logger::write_error_log(&panic_info);
                }
            }
            ::aviutl2::module::ScriptModuleFunctionCallback {
                func: __script_module_callback_trampoline,
                userdata: __userdata,
            }
        }
    })
}

fn parse_params(
    expr: &syn::ExprClosure,
) -> Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>), proc_macro2::TokenStream>
{
    let mut param_bridges = Vec::new();
    let mut param_names = Vec::new();

    for (idx, input) in expr.inputs.iter().enumerate() {
        let syn::Pat::Type(pat_type) = input else {
            return Err(syn::Error::new_spanned(
                input,
                "`script_module_callback` parameters must have explicit type annotations",
            )
            .to_compile_error());
        };
        let pat = &pat_type.pat;
        let ty = &pat_type.ty;
        let syn::Pat::Ident(pat_ident) = pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                pat,
                "`script_module_callback` parameters must be identifiers",
            )
            .to_compile_error());
        };
        let ident = &pat_ident.ident;
        param_names.push(quote::quote! { #ident });
        param_bridges.push(quote::quote! {
            let #pat: #ty = match <#ty as ::aviutl2::module::FromScriptModuleParam>::from_param(__handle, #idx) {
                ::std::option::Option::Some(value) => value,
                ::std::option::Option::None => {
                    let _ = __handle.set_error(&format!(
                        "Failed to convert parameter #{} to {}",
                        #idx,
                        stringify!(#ty)
                    ));
                    return;
                }
            };
        });
    }

    Ok((param_bridges, param_names))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn normal_callback() {
        let input = quote::quote! {
            |value: i32| -> i32 {
                value + 1
            }
        };
        let output = script_module_callback(input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn move_callback() {
        let input = quote::quote! {
            move |value: i32| -> i32 {
                value + offset
            }
        };
        let output = script_module_callback(input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    #[test]
    fn direct_callback() {
        let input = quote::quote! {
            move |handle: &mut ::aviutl2::module::ScriptModuleCallHandle| {
                let value: i32 = handle.get_param(0).unwrap_or(0);
                let _ = handle.push_result(value + 1);
            }
        };
        let output = script_module_direct_callback(input).unwrap();
        insta::assert_snapshot!(format_tokens(output));
    }

    fn format_tokens(tokens: proc_macro2::TokenStream) -> String {
        let wrapped = format!("fn main() {}", tokens);
        let replaced = proc_macro2::TokenStream::from_str(&wrapped).unwrap();
        let formatted = rustfmt_wrapper::rustfmt(replaced).unwrap();
        formatted
            .trim_start_matches("fn main() ")
            .trim()
            .to_string()
    }
}
