use quote::ToTokens;

pub fn generic_menus(
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut item: syn::ItemImpl = syn::parse2(item).map_err(|e| e.to_compile_error())?;
    if item.trait_.is_some() {
        return Err(syn::Error::new_spanned(
            &item,
            "`generic_menus` macro can only be applied to inherent impl blocks",
        )
        .to_compile_error());
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item,
            "`generic_menus` macro does not support generic impl blocks",
        )
        .to_compile_error());
    }
    if item.self_ty.to_token_stream().to_string().contains('<') {
        return Err(syn::Error::new_spanned(
            &item,
            "`generic_menus` macro does not support generic types",
        )
        .to_compile_error());
    }

    let impl_token = item.self_ty.to_token_stream();

    #[derive(Clone, Copy)]
    enum ErrorMode {
        Ignore,
        Log,
        Alert,
    }

    struct Entry {
        is_export: bool,
        menu_name: String,
        method_ident: syn::Ident,
        wrapper_ident: syn::Ident,
        has_self: bool,
        self_is_mut: bool,
        error_mode: ErrorMode,
    }

    let mut entries: Vec<Entry> = Vec::new();

    for it in item.items.iter_mut() {
        let syn::ImplItem::Fn(method) = it else {
            continue;
        };
        let method_ident = method.sig.ident.clone();
        let mut is_import = None::<usize>;
        let mut is_export = None::<usize>;
        for (idx, attr) in method.attrs.iter().enumerate() {
            if attr.path().is_ident("import") {
                is_import = Some(idx);
            }
            if attr.path().is_ident("export") {
                is_export = Some(idx);
            }
        }
        let (kind_idx, is_export_flag) = match (is_import, is_export) {
            (Some(i), None) => (i, false),
            (None, Some(i)) => (i, true),
            (None, None) => continue,
            _ => {
                return Err(syn::Error::new_spanned(
                    &method.sig.ident,
                    "method cannot have both #[import] and #[export]",
                )
                .to_compile_error());
            }
        };

        // Parse name = "...", error = "alert"|"log"
        let attr = method.attrs.remove(kind_idx);
        let mut menu_name: Option<String> = None;
        let mut error_mode = ErrorMode::Alert;
        attr
            .parse_nested_meta(|m| {
                if m.path.is_ident("name") {
                    let value: syn::LitStr = m.value()?.parse()?;
                    menu_name = Some(value.value());
                    Ok(())
                } else if m.path.is_ident("error") {
                    let value: syn::LitStr = m.value()?.parse()?;
                    match value.value().as_str() {
                        "alert" => error_mode = ErrorMode::Alert,
                        "log" => error_mode = ErrorMode::Log,
                        "ignore" => error_mode = ErrorMode::Ignore,
                        _ => return Err(m.error("expected \"alert\", \"log\", or \"ignore\"")),
                    }
                    Ok(())
                } else {
                    Err(m.error("expected `name` or `error`"))
                }
            })
            .map_err(|e| e.to_compile_error())?;
        let menu_name = menu_name.unwrap_or_else(|| method_ident.to_string());

        let mut has_self = false;
        let mut self_is_mut = false;
        for p in method.sig.inputs.iter() {
            if let syn::FnArg::Receiver(r) = p {
                if r.reference.is_none() {
                    return Err(
                        syn::Error::new_spanned(r, "method receiver must be a reference")
                            .to_compile_error(),
                    );
                }
                has_self = true;
                self_is_mut = r.mutability.is_some();
            }
        }
        let wrapper_ident =
            syn::Ident::new(&format!("bridge_{}", method_ident), method_ident.span());

        entries.push(Entry {
            is_export: is_export_flag,
            menu_name,
            method_ident,
            wrapper_ident,
            has_self,
            self_is_mut,
            error_mode,
        });
    }

    // Build registration lines and wrapper fn bodies
    let mut register_lines: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut wrappers: Vec<proc_macro2::TokenStream> = Vec::new();
    for e in entries.iter() {
        let name_str = &e.menu_name;
        let method_ident = &e.method_ident;
        let wrapper_ident = &e.wrapper_ident;
        let reg = if e.is_export {
            quote::quote! { host.register_export_menu(#name_str, #wrapper_ident); }
        } else {
            quote::quote! { host.register_import_menu(#name_str, #wrapper_ident); }
        };
        register_lines.push(reg);

        let call_on_error = match e.error_mode {
            ErrorMode::Ignore => quote::quote! { let _ = ret; },
            ErrorMode::Log => quote::quote! { edit.__output_log_if_error(ret); },
            ErrorMode::Alert => quote::quote! { edit.__alert_if_error(ret); },
        };

        let wrapper = if e.has_self {
            if e.self_is_mut {
                quote::quote! {
                    extern "C" fn #wrapper_ident(edit: *mut ::aviutl2::sys::plugin2::EDIT_SECTION) {
                        let mut edit = unsafe { ::aviutl2::generic::EditSection::from_ptr(edit) };
                        <#impl_token as ::aviutl2::generic::GenericPlugin>::with_instance_mut(|__self| {
                            let ret = <#impl_token>::#method_ident(__self, &mut edit);
                            #call_on_error
                        });
                    }
                }
            } else {
                quote::quote! {
                    extern "C" fn #wrapper_ident(edit: *mut ::aviutl2::sys::plugin2::EDIT_SECTION) {
                        let mut edit = unsafe { ::aviutl2::generic::EditSection::from_ptr(edit) };
                        <#impl_token as ::aviutl2::generic::GenericPlugin>::with_instance(|__self| {
                            let ret = <#impl_token>::#method_ident(__self, &mut edit);
                            #call_on_error
                        });
                    }
                }
            }
        } else {
            quote::quote! {
                extern "C" fn #wrapper_ident(edit: *mut ::aviutl2::sys::plugin2::EDIT_SECTION) {
                    let mut edit = unsafe { ::aviutl2::generic::EditSection::from_ptr(edit) };
                    let ret = <#impl_token>::#method_ident(&mut edit);
                    #call_on_error
                }
            }
        };
        wrappers.push(wrapper);
    }

    Ok(quote::quote! {
        #item

        ::aviutl2::__internal_module! {
            impl ::aviutl2::generic::GenericPluginMenus for #impl_token {
                fn register_menus(host: &mut ::aviutl2::generic::HostAppHandle) {
                    #(#register_lines)*
                    return;

                    #(#wrappers)*
                }
            }
        }
    })
}
