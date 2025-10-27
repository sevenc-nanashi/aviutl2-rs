pub fn plugin(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let attr: syn::Ident = syn::parse2(attr).map_err(|e| e.to_compile_error())?;
    let ast: syn::ItemStruct = syn::parse2(item.clone()).map_err(|e| e.to_compile_error())?;
    let struct_name = &ast.ident;
    Ok(quote::quote! {
        #item

        impl ::aviutl2::__internal_base::singleton_traits::#attr for #struct_name {
            fn get_singleton_state() -> &'static
                ::std::sync::RwLock<::std::option::Option<::aviutl2::__internal_base::state::#attr<#struct_name>>>
            {
                static PLUGIN: ::std::sync::RwLock<Option<::aviutl2::__internal_base::state::#attr<#struct_name>>> =
                    ::std::sync::RwLock::new(None);
                &PLUGIN
            }
        }
    })
}
