use quote::ToTokens;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ErrorMode {
    Ignore,
    Log,
    Alert,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryType {
    Import,
    Export,
    Layer,
    Object,
    Edit,
    Config,
}

struct Entry {
    entry_type: EntryType,
    menu_name: String,
    method_ident: syn::Ident,
    wrapper_ident: syn::Ident,
    has_self: bool,
    self_is_mut: bool,
    error_mode: ErrorMode,
}

fn parse_menu_attr(
    attr: syn::Attribute,
    default_name: &str,
) -> Result<(String, ErrorMode), proc_macro2::TokenStream> {
    let mut name: Option<String> = None;
    let mut error_mode = ErrorMode::Alert;
    attr.parse_nested_meta(|m| {
        if m.path.is_ident("name") {
            let value: syn::LitStr = m.value()?.parse()?;
            name = Some(value.value());
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
    Ok((name.unwrap_or_else(|| default_name.to_string()), error_mode))
}

fn analyze_receiver(sig: &syn::Signature) -> Result<(bool, bool), proc_macro2::TokenStream> {
    let mut has_self = false;
    let mut self_is_mut = false;
    for p in sig.inputs.iter() {
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
    Ok((has_self, self_is_mut))
}

pub fn generic_menus(
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut item: syn::ItemImpl = syn::parse2(item).map_err(|e| e.to_compile_error())?;

    // Validate impl target
    if item.trait_.is_some() {
        return Err(syn::Error::new_spanned(
            &item,
            "`generic_menus` macro can only be applied to inherent impl blocks",
        )
        .to_compile_error());
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "`generic_menus` macro does not support generic impl blocks",
        )
        .to_compile_error());
    }
    if has_generic_args_in_type(&item.self_ty) {
        return Err(syn::Error::new_spanned(
            &item.self_ty,
            "`generic_menus` macro does not support generic types",
        )
        .to_compile_error());
    }

    let impl_token = item.self_ty.to_token_stream();

    let mut entries: Vec<Entry> = Vec::new();
    for it in item.items.iter_mut() {
        let syn::ImplItem::Fn(method) = it else {
            continue;
        };

        let method_ident = method.sig.ident.clone();
        let (attr_idx, entry_type) = match find_menu_attr(&method.attrs) {
            Ok(Some(v)) => v,
            Ok(None) => {
                return Err(syn::Error::new_spanned(
                    &method.sig.ident,
                    "method must have one of #[import], #[export], #[layer], or #[object]",
                )
                .to_compile_error());
            }
            Err(e) => return Err(e),
        };

        // Take and parse attribute
        let attr = method.attrs.remove(attr_idx);
        let (menu_name, error_mode) = parse_menu_attr(attr, &method_ident.to_string())?;

        // Analyze receiver
        let (has_self, self_is_mut) = analyze_receiver(&method.sig)?;
        let wrapper_ident =
            syn::Ident::new(&format!("bridge_{}", method_ident), method_ident.span());

        entries.push(Entry {
            entry_type,
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

        let reg = match e.entry_type {
            EntryType::Export => {
                quote::quote! { host.register_export_menu(#name_str, #wrapper_ident); }
            }
            EntryType::Import => {
                quote::quote! { host.register_import_menu(#name_str, #wrapper_ident); }
            }
            EntryType::Layer => {
                quote::quote! { host.register_layer_menu(#name_str, #wrapper_ident); }
            }
            EntryType::Object => {
                quote::quote! { host.register_object_menu(#name_str, #wrapper_ident); }
            }
            EntryType::Edit => {
                quote::quote! { host.register_edit_menu(#name_str, #wrapper_ident); }
            }
            EntryType::Config => {
                quote::quote! { host.register_config_menu(#name_str, #wrapper_ident); }
            }
        };
        register_lines.push(reg);

        let call_on_error = match e.error_mode {
            ErrorMode::Ignore => quote::quote! { let _ = ret; },
            ErrorMode::Log => quote::quote! { ::aviutl2::generic::__output_log_if_error(ret); },
            ErrorMode::Alert => quote::quote! { ::aviutl2::generic::__alert_if_error(ret); },
        };

        let wrapper = if e.has_self {
            let with_fn = if e.self_is_mut {
                quote::quote!(with_instance_mut)
            } else {
                quote::quote!(with_instance)
            };
            if e.entry_type == EntryType::Config {
                quote::quote! {
                    extern "C" fn #wrapper_ident(hwnd: ::aviutl2::sys::plugin2::HWND, hinstance: ::aviutl2::sys::plugin2::HINSTANCE) {
                        let mut rwh = unsafe { ::aviutl2::generic::__internal_rwh_from_raw(hwnd, hinstance) };
                        <#impl_token as ::aviutl2::generic::GenericPlugin>::#with_fn(|__self| {
                            let ret = <#impl_token>::#method_ident(__self, rwh);
                            #call_on_error
                        });
                    }
                }
            } else {
                quote::quote! {
                    extern "C" fn #wrapper_ident(edit: *mut ::aviutl2::sys::plugin2::EDIT_SECTION) {
                        let mut edit = unsafe { ::aviutl2::generic::EditSection::from_ptr(edit) };
                        <#impl_token as ::aviutl2::generic::GenericPlugin>::#with_fn(|__self| {
                            let ret = <#impl_token>::#method_ident(__self, &mut edit);
                            #call_on_error
                        });
                    }
                }
            }
        } else if e.entry_type == EntryType::Config {
            quote::quote! {
                extern "C" fn #wrapper_ident(hwnd: ::aviutl2::sys::plugin2::HWND, hinstance: ::aviutl2::sys::plugin2::HINSTANCE) {
                    let mut rwh = unsafe { ::aviutl2::generic::__internal_rwh_from_raw(hwnd, hinstance) };
                    let ret = <#impl_token>::#method_ident(rwh);
                    #call_on_error
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

fn has_generic_args_in_type(ty: &syn::Type) -> bool {
    use syn::{PathArguments, Type};
    match ty {
        Type::Path(p) => p
            .path
            .segments
            .iter()
            .any(|seg| !matches!(seg.arguments, PathArguments::None)),
        Type::Reference(r) => has_generic_args_in_type(&r.elem),
        Type::Ptr(p) => has_generic_args_in_type(&p.elem),
        _ => false,
    }
}

fn find_menu_attr(
    attrs: &[syn::Attribute],
) -> Result<Option<(usize, EntryType)>, proc_macro2::TokenStream> {
    static RECOGNIZED_ATTRS: &[&str] = &["import", "export", "layer", "object", "edit", "config"];
    let mut found: Option<(usize, EntryType)> = None;
    for (idx, attr) in attrs.iter().enumerate() {
        for &recognized in RECOGNIZED_ATTRS {
            if attr.path().is_ident(recognized) {
                let entry_type = match recognized {
                    "import" => EntryType::Import,
                    "export" => EntryType::Export,
                    "layer" => EntryType::Layer,
                    "object" => EntryType::Object,
                    "edit" => EntryType::Edit,
                    "config" => EntryType::Config,
                    _ => unreachable!(),
                };
                if found.is_some() {
                    return Err(syn::Error::new_spanned(
                        &attrs[0],
                        "method can have only one of #[import], #[export], #[layer], #[object], or #[edit]",
                    )
                    .to_compile_error());
                }
                found = Some((idx, entry_type));
            }
        }
    }
    Ok(found)
}
