struct BaseEnumVariant {
    ident: syn::Ident,
    name: Option<String>,
    discriminant: Option<syn::Expr>,
}
struct EnumVariant {
    ident: syn::Ident,
    name: String,
    discriminant: syn::Expr,
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
    let variants = parse_enum_variants(&variants).map_err(|e| e.to_compile_error())?;

    let to_select_items = impl_to_select_items(&variants)?;
    let from_select_item_value = impl_from_select_item_value(&name, &variants)?;
    let to_select_item_value = impl_to_select_item_value(&variants)?;

    let expanded = quote::quote! {
        impl ::aviutl2::filter::FilterConfigSelectItems for #name {
            #to_select_items
            #from_select_item_value
            #to_select_item_value
        }
    };

    Ok(expanded)
}

fn parse_enum_variant(variant: &syn::Variant) -> Result<BaseEnumVariant, syn::Error> {
    let ident = variant.ident.clone();
    let name = variant
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("item"))
        .and_then(|attr| {
            attr.parse_args_with(|input: syn::parse::ParseStream| {
                let name: syn::LitStr = input.parse()?;
                Ok(name.value())
            })
            .ok()
        });

    let discriminant = variant.discriminant.as_ref().map(|(_, expr)| expr.clone());
    if !variant.fields.is_empty() {
        return Err(syn::Error::new_spanned(
            variant,
            "Enum variants must be unit-like (no fields)",
        ));
    }

    Ok(BaseEnumVariant {
        ident,
        name,
        discriminant,
    })
}

fn parse_enum_variants(variants: &[BaseEnumVariant]) -> Result<Vec<EnumVariant>, syn::Error> {
    let mut result = Vec::new();
    let mut last_value = None;

    for variant in variants {
        let name = if let Some(name) = &variant.name {
            name.clone()
        } else {
            variant.ident.to_string()
        };
        let discriminant = if let Some(discriminant) = &variant.discriminant {
            discriminant.clone()
        } else if let Some(last) = &last_value {
            syn::parse_quote! { #last + 1 }
        } else {
            syn::parse_quote! { 0 }
        };
        last_value = Some(discriminant.clone());
        result.push(EnumVariant {
            ident: variant.ident.clone(),
            name,
            discriminant,
        });
    }

    Ok(result)
}

fn impl_to_select_items(
    variants: &[EnumVariant],
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut items = Vec::new();

    for variant in variants {
        let name = &variant.name;
        let discriminant = &variant.discriminant;
        items.push(quote::quote! {
            ::aviutl2::filter::FilterConfigSelectItem {
                name: #name.to_string(),
                value: #discriminant,
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
        let discriminant = &variant.discriminant;
        match_arms.push(quote::quote! {
            else if value == (const { #discriminant }) {
                return Self::#ident;
            }
        });
    }

    let expanded = quote::quote! {
        fn from_select_item_value(value: i32) -> Self {
            if false {
                unreachable!()
            }
            #(#match_arms)*
            else {
                panic!("Invalid value for {}", stringify!(#enum_name))
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
        let discriminant = &variant.discriminant;
        match_arms.push(quote::quote! {
            Self::#ident => (const { #discriminant }),
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
        #[item(name = "Hoge")]
        Hoge,
        #[item(name = "Fuga")]
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
