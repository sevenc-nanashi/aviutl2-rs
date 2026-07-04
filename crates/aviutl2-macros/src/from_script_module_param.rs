pub fn from_script_module_param(
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
                "`FromScriptModuleParam` can only be derived for structs with named fields",
            )
            .to_compile_error());
        }
    };

    let field_initializers = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let ty = &field.ty;
        quote::quote! {
            #field_name: <#ty as ::aviutl2::module::FromScriptModuleParamTable>::from_param_table(&table, #field_name_str)
                .map_err(|error| {
                    ::aviutl2::module::GetParamError::ConversionError(
                        ::aviutl2::module::ParamConversionError::new(format!(
                            "field `{}`: {}",
                            #field_name_str,
                            error
                        ))
                    )
                })?
        }
    });

    let expanded = quote::quote! {
        impl<'a> ::aviutl2::module::FromScriptModuleParam<'a> for #ident {
            type Error = ::aviutl2::module::ParamConversionError;

            fn from_param(
                param: &'a ::aviutl2::module::ScriptModuleCallHandle,
                index: usize,
            ) -> ::aviutl2::module::GetParamResult<Self, Self::Error> {
                let table = ::aviutl2::module::ScriptModuleParamTable::from_param(param, index)
                    .map_err(|error| {
                        ::aviutl2::module::GetParamError::ConversionError(
                            ::aviutl2::module::ParamConversionError::new(error.to_string())
                        )
                    })?;
                Ok(Self {
                    #(#field_initializers),*
                })
            }
        }
    };

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_from_script_module_param() {
        let input = quote::quote! {
            struct MyParam {
                string_value: String,
                int_value: i32,
            }
        };
        let output = super::from_script_module_param(input).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }
}
