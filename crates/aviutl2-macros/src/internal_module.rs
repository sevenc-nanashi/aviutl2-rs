pub fn internal_module(
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let random_ident = syn::Ident::new(
        &format!("__aviutl2_internal_{}", rand::random::<u64>()),
        proc_macro2::Span::call_site(),
    );
    let expanded = quote::quote! {
        #[doc(hidden)]
        mod #random_ident {
            #item
        }
    };

    Ok(expanded)
}
