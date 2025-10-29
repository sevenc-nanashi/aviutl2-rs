pub fn into_script_module_return_value(
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let ast: syn::DeriveInput = syn::parse2(item).map_err(|e| e.to_compile_error())?;
    let ident = &ast.ident;

    let fields = match ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ast,
                "`IntoScriptModuleReturnValue` can only be derived for structs with named fields",
            )
            .to_compile_error());
        }
    };

    let push_fields = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote::quote! {
            if let ::std::option::Option::Some(value) = ::aviutl2::module::table_converter::ToOptionalTableEntry::to_optional(&self.#field_name) {
               map.insert(
                    ::std::string::String::from(stringify!(#field_name)),
                    value,
                );
            }
        }
    });

    let expanded = quote::quote! {
        impl ::aviutl2::module::IntoScriptModuleReturnValue for #ident {
            fn into_return_values(self) -> ::aviutl2::AnyResult<Vec<::aviutl2::module::ScriptModuleReturnValue>> {
                let mut map = ::std::collections::HashMap::new();
                #(#push_fields)*
                ::aviutl2::module::IntoScriptModuleReturnValue::into_return_values(map)
            }
        }
    };

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_to_script_module_return_value() {
        let input = quote::quote! {
            struct MyReturnValue {
                string_value: String,
                string_option: Option<String>,
            }
        };
        let output = super::into_script_module_return_value(input).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }
}
