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

    struct Entry {
        is_export: bool,
        menu_name: String,
        method_ident: syn::Ident,
        wrapper_ident: syn::Ident,
        has_self: bool,
    }

    let mut entries: Vec<Entry> = Vec::new();

    for it in item.items.iter_mut() {
        let syn::ImplItem::Fn(method) = it else { continue; };
        let method_ident = method.sig.ident.clone();
        let mut is_import = None::<usize>;
        let mut is_export = None::<usize>;
        for (idx, attr) in method.attrs.iter().enumerate() {
            if attr.path().is_ident("import") { is_import = Some(idx); }
            if attr.path().is_ident("export") { is_export = Some(idx); }
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

        // Parse name = "..."
        let attr = method.attrs.remove(kind_idx);
        let mut menu_name: Option<String> = None;
        attr.parse_nested_meta(|m| {
            if m.path.is_ident("name") {
                let value: syn::LitStr = m.value()?.parse()?;
                menu_name = Some(value.value());
                Ok(())
            } else {
                Err(m.error("expected `name`"))
            }
        }).map_err(|e| e.to_compile_error())?;
        let menu_name = menu_name.unwrap_or_else(|| method_ident.to_string());

        let has_self = method
            .sig
            .inputs
            .iter()
            .any(|p| matches!(p, syn::FnArg::Receiver(_)));
        let wrapper_ident = syn::Ident::new(&format!("bridge_{}", method_ident), method_ident.span());

        entries.push(Entry {
            is_export: is_export_flag,
            menu_name,
            method_ident,
            wrapper_ident,
            has_self,
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

        let wrapper = if e.has_self {
            quote::quote! {
                extern "C" fn #wrapper_ident(edit: *mut ::aviutl2::sys::plugin2::EDIT_SECTION) {
                    let mut edit = unsafe { ::aviutl2::generic::EditSection::from_ptr(edit) };
                    let __state = <#impl_token as ::aviutl2::generic::__bridge::GenericSingleton>::__get_singleton_state();
                    let __state = __state.read().expect("Plugin handle is not initialized");
                    let __self = &__state.as_ref().expect("Plugin instance is not initialized").instance;
                    let _ = <#impl_token>::#method_ident(__self, &mut edit);
                }
            }
        } else {
            quote::quote! {
                extern "C" fn #wrapper_ident(edit: *mut ::aviutl2::sys::plugin2::EDIT_SECTION) {
                    let mut edit = unsafe { ::aviutl2::generic::EditSection::from_ptr(edit) };
                    let _ = <#impl_token>::#method_ident(&mut edit);
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

