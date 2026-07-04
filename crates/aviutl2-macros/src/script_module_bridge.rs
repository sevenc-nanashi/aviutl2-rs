use quote::ToTokens;
use syn::parse::Parser;

pub fn parse_unwind_attr(attr: proc_macro2::TokenStream) -> Result<bool, proc_macro2::TokenStream> {
    let mut unwind = true;
    if attr.is_empty() {
        return Ok(unwind);
    }
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("unwind") {
            if meta.input.is_empty() {
                unwind = true;
                return Ok(());
            }
            let value: syn::LitBool = meta.value()?.parse()?;
            unwind = value.value;
            Ok(())
        } else {
            Err(meta.error("expected `unwind`"))
        }
    });
    parser.parse2(attr).map_err(|e| e.to_compile_error())?;
    Ok(unwind)
}

pub fn parse_inherent_impl(
    item: proc_macro2::TokenStream,
    macro_name: &str,
) -> Result<syn::ItemImpl, proc_macro2::TokenStream> {
    let item: syn::ItemImpl = syn::parse2(item).map_err(|e| e.to_compile_error())?;
    if item.trait_.is_some() {
        return Err(syn::Error::new_spanned(
            item,
            format!("`{macro_name}` macro can only be applied to inherent impl blocks"),
        )
        .to_compile_error());
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            item,
            format!("`{macro_name}` macro does not support generic impl blocks"),
        )
        .to_compile_error());
    }
    if item.self_ty.to_token_stream().to_string().contains('<') {
        return Err(syn::Error::new_spanned(
            item,
            format!("`{macro_name}` macro does not support generic types"),
        )
        .to_compile_error());
    }
    Ok(item)
}

pub struct MethodBridge {
    pub method_name_str: String,
    pub internal_method_name: syn::Ident,
    pub body: proc_macro2::TokenStream,
}

pub enum ReceiverKind {
    ScriptModuleSingleton,
    UserData,
}

pub fn create_method_bridge(
    impl_token: &proc_macro2::TokenStream,
    method: &mut syn::ImplItemFn,
    receiver_kind: ReceiverKind,
) -> Result<MethodBridge, proc_macro2::TokenStream> {
    let method_name = method.sig.ident.clone();
    let method_name_str = method_name.to_string();
    let internal_method_name =
        syn::Ident::new(&format!("bridge_{}", method_name), method_name.span());

    let direct_index = method
        .attrs
        .iter()
        .position(|attr| attr.path().is_ident("direct"));
    let body = if let Some(direct_index) = direct_index {
        method.attrs.remove(direct_index);
        create_direct_body(impl_token, method, &receiver_kind)?
    } else {
        create_converted_body(impl_token, method, &receiver_kind)?
    };

    Ok(MethodBridge {
        method_name_str,
        internal_method_name,
        body,
    })
}

pub fn wrap_with_unwind(
    internal_method_name: &syn::Ident,
    method_name_str: &str,
    body: &proc_macro2::TokenStream,
    unsafe_extern: bool,
    unwind: bool,
) -> proc_macro2::TokenStream {
    let extern_safety = if unsafe_extern {
        quote::quote! { unsafe }
    } else {
        quote::quote! {}
    };
    if unwind {
        quote::quote! {
            #extern_safety extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                if let Err(panic_info) = ::aviutl2::__catch_unwind_with_panic_info(|| {
                    #body
                }) {
                    ::aviutl2::tracing::error!(
                        "Panic occurred during {}: {}",
                        #method_name_str,
                        panic_info
                    );
                    let _ = ::aviutl2::logger::write_error_log(&panic_info);
                }
            }
        }
    } else {
        quote::quote! {
            #extern_safety extern "C" fn #internal_method_name(smp: *mut ::aviutl2::sys::module2::SCRIPT_MODULE_PARAM) {
                #body
            }
        }
    }
}

fn create_direct_body(
    impl_token: &proc_macro2::TokenStream,
    method: &syn::ImplItemFn,
    receiver_kind: &ReceiverKind,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let method_name = &method.sig.ident;
    let receiver = parse_receiver(method)?;

    Ok(match receiver_kind {
        ReceiverKind::ScriptModuleSingleton => match receiver {
            MethodReceiver::None => quote::quote! {
                let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
                let () = <#impl_token>::#method_name(&mut __handle);
            },
            MethodReceiver::Shared => quote::quote! {
                let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance(|__internal_self| {
                    let () = <#impl_token>::#method_name(__internal_self, &mut __handle);
                });
            },
            MethodReceiver::Mutable => quote::quote! {
                let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance_mut(|__internal_self| {
                    let () = <#impl_token>::#method_name(__internal_self, &mut __handle);
                });
            },
        },
        ReceiverKind::UserData => {
            let call_body = match receiver {
                MethodReceiver::None => {
                    quote::quote! { let () = <#impl_token>::#method_name(&mut __handle); }
                }
                MethodReceiver::Shared | MethodReceiver::Mutable => {
                    quote::quote! { let () = <#impl_token>::#method_name(__internal_self, &mut __handle); }
                }
            };
            create_userdata_call_body(impl_token, method_name, call_body, receiver)
        }
    })
}

fn create_converted_body(
    impl_token: &proc_macro2::TokenStream,
    method: &syn::ImplItemFn,
    receiver_kind: &ReceiverKind,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let method_name = &method.sig.ident;
    let params = &method.sig.inputs;
    let mut param_bridges = Vec::new();
    let mut param_names = Vec::new();
    let mut param_index: usize = 0;
    let mut receiver = MethodReceiver::None;

    for param in params.iter() {
        match param {
            syn::FnArg::Receiver(r) => {
                if r.reference.is_none() {
                    return Err(
                        syn::Error::new_spanned(r, "method receiver must be a reference")
                            .to_compile_error(),
                    );
                }
                receiver = if r.mutability.is_some() {
                    MethodReceiver::Mutable
                } else {
                    MethodReceiver::Shared
                };
                param_names.push(quote::quote! { __internal_self });
            }
            syn::FnArg::Typed(pat_type) => {
                let ty = &pat_type.ty;
                let pat = &pat_type.pat;
                let idx = param_index;
                param_bridges.push(quote::quote! {
                    let #pat: #ty = match <#ty as ::aviutl2::module::FromScriptModuleParam>::from_param(&__handle, #idx) {
                        ::std::result::Result::Ok(value) => value,
                        ::std::result::Result::Err(error) => {
                            let _ = __handle.set_error(&format!(
                                "Failed to convert parameter #{} to {}: {}",
                                #idx,
                                stringify!(#ty),
                                error
                            ));
                            return;
                        }
                    };
                });
                param_names.push(quote::quote! { #pat });
                param_index += 1;
            }
        }
    }

    Ok(match receiver_kind {
        ReceiverKind::ScriptModuleSingleton => match receiver {
            MethodReceiver::None => quote::quote! {
                let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
                #(#param_bridges)*
                let fn_result = <#impl_token>::#method_name(#(#param_names),*);
                ::aviutl2::module::__push_return_value(&mut __handle, fn_result);
            },
            MethodReceiver::Shared => quote::quote! {
                let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance(|__internal_self| {
                    #(#param_bridges)*
                    let fn_result = <#impl_token>::#method_name(#(#param_names),*);
                    ::aviutl2::module::__push_return_value(&mut __handle, fn_result);
                });
            },
            MethodReceiver::Mutable => quote::quote! {
                let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
                <#impl_token as ::aviutl2::module::ScriptModule>::with_instance_mut(|__internal_self| {
                    #(#param_bridges)*
                    let fn_result = <#impl_token>::#method_name(#(#param_names),*);
                    ::aviutl2::module::__push_return_value(&mut __handle, fn_result);
                });
            },
        },
        ReceiverKind::UserData => create_userdata_call_body(
            impl_token,
            method_name,
            quote::quote! {
                #(#param_bridges)*
                let fn_result = <#impl_token>::#method_name(#(#param_names),*);
                ::aviutl2::module::__push_return_value(&mut __handle, fn_result);
            },
            receiver,
        ),
    })
}

fn create_userdata_call_body(
    impl_token: &proc_macro2::TokenStream,
    _method_name: &syn::Ident,
    call_body: proc_macro2::TokenStream,
    receiver: MethodReceiver,
) -> proc_macro2::TokenStream {
    match receiver {
        MethodReceiver::None => quote::quote! {
            let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
            #call_body
        },
        MethodReceiver::Shared => quote::quote! {
            let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
            {
                let __userdata = unsafe {
                    &*((*smp).userdata as *const ::std::sync::Mutex<#impl_token>)
                };
                let __userdata = __userdata
                    .lock()
                    .expect("script module meta table userdata mutex poisoned");
                let __internal_self = &*__userdata;
                #call_body
            }
        },
        MethodReceiver::Mutable => quote::quote! {
            let mut __handle = unsafe { ::aviutl2::module::ScriptModuleCallHandle::from_raw(smp) };
            {
                let __userdata = unsafe {
                    &*((*smp).userdata as *const ::std::sync::Mutex<#impl_token>)
                };
                let mut __userdata = __userdata
                    .lock()
                    .expect("script module meta table userdata mutex poisoned");
                let __internal_self = &mut *__userdata;
                #call_body
            }
        },
    }
}

#[derive(Clone, Copy)]
enum MethodReceiver {
    None,
    Shared,
    Mutable,
}

fn parse_receiver(method: &syn::ImplItemFn) -> Result<MethodReceiver, proc_macro2::TokenStream> {
    let mut receiver = MethodReceiver::None;
    for param in method.sig.inputs.iter() {
        if let syn::FnArg::Receiver(r) = param {
            if r.reference.is_none() {
                return Err(
                    syn::Error::new_spanned(r, "method receiver must be a reference")
                        .to_compile_error(),
                );
            }
            receiver = if r.mutability.is_some() {
                MethodReceiver::Mutable
            } else {
                MethodReceiver::Shared
            };
        }
    }
    Ok(receiver)
}
