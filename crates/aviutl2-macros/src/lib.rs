use quote::ToTokens;

enum FilterConfigField {
    Track {
        id: String,
        name: String,
        default: f64,
        min: f64,
        max: f64,
        step: f64,
    },
    Check {
        id: String,
        name: String,
        default: bool,
    },
    Color {
        id: String,
        name: String,
        default: u32,
    },
    Select {
        id: String,
        name: String,
        default: i32,
        options: Vec<String>,
    },
    File {
        id: String,
        name: String,
        filter: String,
    },
}

#[proc_macro_attribute]
pub fn filter_config(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = syn::parse_macro_input!(item as syn::ItemStruct);
    let fields = item
        .fields
        .iter_mut()
        .map(|f| filter_config_field(f))
        .collect::<Result<Vec<_>, _>>();

    let fields = match fields {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };

    let generated = quote::quote! {
        #item
    };
    generated.into()
}

fn filter_config_field(field: &mut syn::Field) -> Result<FilterConfigField, syn::Error> {
    let attrs = &mut field.attrs;
    let mut recognized_fields = vec![];
    static RECOGNIZED_FIELDS: &[&str] = &["track", "check", "color", "select", "file"];
    attrs.retain(|attr| {
        if attr
            .path()
            .get_ident()
            .is_some_and(|id| RECOGNIZED_FIELDS.contains(&id.to_string().as_str()))
        {
            recognized_fields.push(attr.clone());
            false
        } else {
            true
        }
    });
    if recognized_fields.len() != 1 {
        return Err(syn::Error::new_spanned(
            field,
            format!(
                "Exactly one of #[track], #[check], #[color], #[select], or #[file] is required (found {})",
                recognized_fields.len()
            ),
        ));
    }
    let recognized_attr = recognized_fields.into_iter().next().unwrap();
    match recognized_attr
        .path()
        .get_ident()
        .unwrap()
        .to_string()
        .as_str()
    {
        "track" => {
            let mut name = None;
            let mut default = None;
            let mut min = None;
            let mut max = None;
            let mut step = None;

            recognized_attr.parse_nested_meta(|m| {
                if m.path.is_ident("name") {
                    name = Some(m.value()?.parse::<syn::LitStr>()?);
                } else if m.path.is_ident("default") {
                    default = Some(m.value()?.parse::<syn::LitFloat>()?);
                } else if m.path.is_ident("min") {
                    min = Some(m.value()?.parse::<syn::LitFloat>()?);
                } else if m.path.is_ident("max") {
                    max = Some(m.value()?.parse::<syn::LitFloat>()?);
                } else if m.path.is_ident("step") {
                    step = Some(m.value()?.parse::<syn::LitFloat>()?);
                } else {
                    return Err(m.error("Unknown attribute for track"));
                }
                Ok(())
            })?;

            let name = name
                .ok_or_else(|| syn::Error::new_spanned(&recognized_attr, "name is required"))?
                .value();
            let default = default
                .ok_or_else(|| syn::Error::new_spanned(&recognized_attr, "default is required"))?
                .base10_parse::<f64>()?;
            let min = min
                .ok_or_else(|| syn::Error::new_spanned(&recognized_attr, "min is required"))?
                .base10_parse::<f64>()?;
            let max = max
                .ok_or_else(|| syn::Error::new_spanned(&recognized_attr, "max is required"))?
                .base10_parse::<f64>()?;
            let step = step
                .ok_or_else(|| syn::Error::new_spanned(&recognized_attr, "step is required"))?
                .base10_parse::<f64>()?;

            if !(min <= default && default <= max) {
                return Err(syn::Error::new_spanned(
                    &recognized_attr,
                    "default must be between min and max",
                ));
            }
            Ok(FilterConfigField::Track {
                id: field.ident.as_ref().unwrap().to_string(),
                name,
                default,
                min,
                max,
                step,
            })
        }

        _ => unreachable!(),
    }
}
