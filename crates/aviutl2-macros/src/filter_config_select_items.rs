struct EnumVariant {
    ident: syn::Ident,
    name: String,
}

pub fn filter_config_select_items(
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let item: syn::ItemEnum = syn::parse2(item).map_err(|e| e.to_compile_error())?;
    let name = item.ident.clone();
    let variants = item
        .variants
        .iter()
        .map(parse_enum_variant)
        .collect::<crate::utils::CombinedVecResults<_>>()
        .into_result()?;
    if variants.is_empty() {
        return Err(
            syn::Error::new_spanned(item, "Enum must have at least one variant")
                .into_compile_error(),
        );
    }
    let to_select_items = impl_to_select_items(&variants)?;
    let from_select_item_value = impl_from_select_item_value(&name, &variants)?;
    let to_select_item_value = impl_to_select_item_value(&variants)?;

    let expanded = quote::quote! {
        #[automatically_derived]
        impl ::aviutl2::filter::FilterConfigSelectItems for #name {
            #to_select_items
            #from_select_item_value
            #to_select_item_value
        }
    };

    Ok(expanded)
}

fn parse_enum_variant(variant: &syn::Variant) -> Result<EnumVariant, syn::Error> {
    let ident = variant.ident.clone();
    let name = variant
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("item"))
        .and_then(|attr| {
            attr.parse_args_with(|input: syn::parse::ParseStream| {
                let lookahead = input.lookahead1();
                if lookahead.peek(syn::Ident) && input.peek2(syn::Token![=]) {
                    let ident: syn::Ident = input.parse()?;
                    if ident != "name" {
                        return Err(syn::Error::new_spanned(ident, "Expected `name`"));
                    }
                    let _eq_token: syn::Token![=] = input.parse()?;
                    let name_lit: syn::LitStr = input.parse()?;
                    Ok(name_lit.value())
                } else {
                    Err(lookahead.error())
                }
            })
            .ok()
        });

    if !variant.fields.is_empty() {
        return Err(syn::Error::new_spanned(
            variant,
            "Enum variants must be unit-like (no fields)",
        ));
    }

    Ok(EnumVariant {
        ident,
        name: name.unwrap_or_else(|| variant.ident.to_string()),
    })
}

fn impl_to_select_items(
    variants: &[EnumVariant],
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut items = Vec::new();

    for variant in variants {
        let name = &variant.name;
        let ident = &variant.ident;
        items.push(quote::quote! {
            ::aviutl2::filter::FilterConfigSelectItem {
                name: #name.to_string(),
                value: Self::#ident as i32,
            }
        });
    }

    let expanded = quote::quote! {
        fn to_select_items() -> Vec<::aviutl2::filter::FilterConfigSelectItem> {
            vec![
                #(#items),*
            ]
        }
    };

    Ok(expanded)
}

fn impl_from_select_item_value(
    enum_name: &syn::Ident,
    variants: &[EnumVariant],
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut match_arms = Vec::new();

    for variant in variants {
        let ident = &variant.ident;
        match_arms.push(quote::quote! {
            _ if value == Self::#ident as i32 => {
                Self::#ident
            }
        });
    }

    let expanded = quote::quote! {
        fn from_select_item_value(value: i32) -> Self {
            match value {
                #(#match_arms)*
                _ => {
                    panic!("Invalid value for {}", stringify!(#enum_name))
                }
            }
        }
    };

    Ok(expanded)
}

fn impl_to_select_item_value(
    variants: &[EnumVariant],
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut match_arms = Vec::new();

    for variant in variants {
        let ident = &variant.ident;
        match_arms.push(quote::quote! {
            Self::#ident => Self::#ident as i32,
        });
    }

    let expanded = quote::quote! {
        fn to_select_item_value(&self) -> i32 {
            match self {
                #(#match_arms)*
            }
        }
    };

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use aviutl2::filter::FilterConfigSelectItems;

    #[derive(Debug, PartialEq, Eq, aviutl2::filter::FilterConfigSelectItems)]
    enum MySelectItem {
        #[item(name = "ほげ")]
        Hoge,
        #[item(name = "ふが")]
        Fuga,

        Foo = 42,
        Bar,
    }

    #[test]
    fn test_select_items() {
        let items = MySelectItem::to_select_items();
        assert_eq!(items.len(), 4);
        insta::assert_debug_snapshot!(items);
    }

    #[test]
    fn test_from_select_item_value() {
        assert_eq!(MySelectItem::from_select_item_value(0), MySelectItem::Hoge);
        assert_eq!(MySelectItem::from_select_item_value(1), MySelectItem::Fuga);
        assert_eq!(MySelectItem::from_select_item_value(42), MySelectItem::Foo);
        assert_eq!(MySelectItem::from_select_item_value(43), MySelectItem::Bar);

        let result = std::panic::catch_unwind(|| MySelectItem::from_select_item_value(2));
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshot() {
        let code = quote::quote! {
            #[derive(Debug, aviutl2::filter::FilterConfigSelectItems)]
            enum MySelectItem {
                #[item(name = "Hoge")]
                Hoge,
                #[item(name = "Fuga")]
                Fuga,

                Foo = 42,
                Bar,
            }
        };
        let output = super::filter_config_select_items(code).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }
}
