use syn::parse::Parse;

pub fn filter_config_items(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut item = syn::parse_macro_input!(item as syn::ItemStruct);

    if let Some(value) = validate_filter_config(&mut item) {
        return value;
    }

    let name = &item.ident;
    let (fields, field_errors) = item
        .fields
        .iter_mut()
        .map(filter_config_field)
        .partition::<Vec<_>, _>(Result::is_ok);
    let field_errors = field_errors
        .into_iter()
        .map(Result::unwrap_err)
        .reduce(|mut a, b| {
            a.combine(b);
            a
        });
    if let Some(err) = field_errors {
        return err.into_compile_error().into();
    }
    let fields = fields.into_iter().map(Result::unwrap).collect::<Vec<_>>();
    let to_config_items = filter_config_items_to_config_items(&fields);
    let from_config_items = filter_config_items_from_filter_config(&fields);

    let expanded = quote::quote! {
        impl ::aviutl2::filter::FilterConfigItems for #name {
            #to_config_items

            #from_config_items
        }
    };

    expanded.into()
}

#[derive(Debug)]
enum FilterConfigField {
    Track {
        id: String,
        name: String,
        default: f64,
        min: f64,
        max: f64,
        step: TrackStep,
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
        items: Vec<String>,
    },
    File {
        id: String,
        name: String,
        filters: Vec<FileFilterEntry>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackStep {
    One,
    PointOne,
    PointZeroOne,
    PointZeroZeroOne,
}

impl std::str::FromStr for TrackStep {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(TrackStep::One),
            "0.1" => Ok(TrackStep::PointOne),
            "0.01" => Ok(TrackStep::PointZeroOne),
            "0.001" => Ok(TrackStep::PointZeroZeroOne),
            _ => Err("expected 1.0, 0.1, 0.01, or 0.001"),
        }
    }
}
impl From<TrackStep> for decimal_rs::Decimal {
    fn from(value: TrackStep) -> Self {
        match value {
            TrackStep::One => "1.0",
            TrackStep::PointOne => "0.1",
            TrackStep::PointZeroOne => "0.01",
            TrackStep::PointZeroZeroOne => "0.001",
        }
        .parse()
        .unwrap()
    }
}

#[derive(Debug)]
struct FileFilterEntry {
    name: String,
    exts: Vec<String>,
}

impl syn::parse::Parse for FileFilterEntry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![=>]>()?;
        let exts: syn::ExprArray = input.parse()?;
        let exts = exts
            .elems
            .iter()
            .map(|e| {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = e
                {
                    Ok(s.value())
                } else {
                    Err(syn::Error::new_spanned(
                        e,
                        "expected string literal for file extension",
                    ))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FileFilterEntry {
            name: name.value(),
            exts,
        })
    }
}

fn filter_config_items_to_config_items(fields: &[FilterConfigField]) -> proc_macro2::TokenStream {
    let to_filter_config_fields = fields
        .iter()
        .map(|f| match f {
            FilterConfigField::Track {
                id: _,
                name,
                default,
                min,
                max,
                step,
            } => {
                let step_enum = match step {
                    TrackStep::One => {
                        quote::quote! { ::aviutl2::filter::FilterConfigTrackStep::One }
                    },
                    TrackStep::PointOne => {
                        quote::quote! { ::aviutl2::filter::FilterConfigTrackStep::PointOne }
                    },
                    TrackStep::PointZeroOne => {
                        quote::quote! { ::aviutl2::filter::FilterConfigTrackStep::PointZeroOne }
                    },
                    TrackStep::PointZeroZeroOne => {
                        quote::quote! { ::aviutl2::filter::FilterConfigTrackStep::PointZeroZeroOne }
                    }
                };
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Track(::aviutl2::filter::FilterConfigTrack {
                        name: #name.to_string(),
                        value: #default,
                        range: #min..=#max,
                        step: #step_enum,
                    })
                }
            }
            FilterConfigField::Check { id: _, name, default } => {
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Checkbox(::aviutl2::filter::FilterConfigCheckbox {
                        name: #name.to_string(),
                        value: #default,
                    })
                }
            }
            FilterConfigField::Color { id: _, name, default } => {
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Color(::aviutl2::filter::FilterConfigColor {
                        name: #name.to_string(),
                        value: #default.into(),
                    })
                }
            }
            FilterConfigField::Select {
                id: _,
                name,
                default,
                items,
            } => {
                let items = items.iter().enumerate().map(|(i, item)| {
                    quote::quote! {
                        ::aviutl2::filter::FilterConfigSelectItem {
                            name: #item.to_string(),
                            value: #i as i32,
                        }
                    }
                });
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Select(::aviutl2::filter::FilterConfigSelect {
                        name: #name.to_string(),
                        value: #default,
                        items: vec![#(#items),*]
                    })
                }
            }
            FilterConfigField::File { id: _, name, filters: filter } => {
                let filter_entries = filter.iter().map(|entry| {
                    let n = &entry.name;
                    let exts = &entry.exts;
                    quote::quote! {
                        ::aviutl2::common::FileFilter {
                            name: #n.to_string(),
                            extensions: vec![#(#exts.to_string()),*],
                        }
                    }
                });
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::File(::aviutl2::filter::FilterConfigFile {
                        name: #name.to_string(),
                        value: String::new(),
                        filters: vec![#(#filter_entries),*],
                    })
                }
            }
    })
        .collect::<Vec<_>>();

    quote::quote! {
        fn to_config_items() -> Vec<::aviutl2::filter::FilterConfigItem> {
            vec![
                #(#to_filter_config_fields),*
            ]
        }
    }
}

fn filter_config_items_from_filter_config(
    config_fields: &[FilterConfigField],
) -> proc_macro2::TokenStream {
    let field_assign = config_fields
        .iter()
        .enumerate()
        .map(|(i, f)| match f {
            FilterConfigField::Track { id, step, .. } => {
                let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
                if *step == TrackStep::One {
                    // i32
                    return quote::quote! {
                        #id_ident: match items[#i] {
                            ::aviutl2::filter::FilterConfigItem::Track(ref track) => (track.value as i32) as _,
                            _ => panic!("Expected Track at index {}", #i),
                        }
                    };
                }
                quote::quote! {
                    #id_ident: match items[#i] {
                        ::aviutl2::filter::FilterConfigItem::Track(ref track) => track.value as _,
                        _ => panic!("Expected Track at index {}", #i),
                    }
                }
            }
            FilterConfigField::Check { id, .. } => {
                let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
                quote::quote! {
                    #id_ident: match items[#i] {
                        ::aviutl2::filter::FilterConfigItem::Checkbox(ref check) => check.value,
                        _ => panic!("Expected Checkbox at index {}", #i),
                    }
                }
            }
            FilterConfigField::Color { id, .. } => {
                let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
                quote::quote! {
                    #id_ident: match items[#i] {
                        ::aviutl2::filter::FilterConfigItem::Color(ref color) => color.value.clone().into(),
                        _ => panic!("Expected Color at index {}", #i),
                    }
                }
            }
            FilterConfigField::Select { id, .. } => {
                let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
                quote::quote! {
                    #id_ident: match items[#i] {
                        ::aviutl2::filter::FilterConfigItem::Select(ref select) => (select.value as usize) as _,
                        _ => panic!("Expected Select at index {}", #i),
                    }
                }
            }
            FilterConfigField::File { id, .. } => {
                let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
                quote::quote! {
                    #id_ident: match items[#i] {
                        ::aviutl2::filter::FilterConfigItem::File(ref file) =>
                            if file.value.is_empty() {
                                None
                            } else {
                                Some(std::path::PathBuf::from(&file.value))
                            },
                        _ => panic!("Expected File at index {}", #i),
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    quote::quote! {
        fn from_config_items(items: &[::aviutl2::filter::FilterConfigItem]) -> Self {
            Self {
                #(
                    #field_assign
                ),*
            }
        }
    }
}

fn validate_filter_config(item: &mut syn::ItemStruct) -> Option<proc_macro::TokenStream> {
    let fields = item
        .fields
        .iter_mut()
        .map(filter_config_field)
        .collect::<Result<Vec<_>, _>>();
    let fields = match fields {
        Ok(f) => f,
        Err(e) => return Some(e.to_compile_error().into()),
    };
    let field_names = fields
        .iter()
        .map(|f| match f {
            FilterConfigField::Track { name, .. } => name,
            FilterConfigField::Check { name, .. } => name,
            FilterConfigField::Color { name, .. } => name,
            FilterConfigField::Select { name, .. } => name,
            FilterConfigField::File { name, .. } => name,
        })
        .collect::<Vec<_>>();

    if field_names.len()
        != field_names
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
    {
        // TODO: フィールドに対してエラーを吐くようにしたい
        return Some(
            syn::Error::new_spanned(&item, "Field names must be unique")
                .to_compile_error()
                .into(),
        );
    }
    None
}

fn filter_config_field(field: &mut syn::Field) -> Result<FilterConfigField, syn::Error> {
    static RECOGNIZED_FIELDS: &[&str] = &["track", "check", "color", "select", "file"];
    let recognized_fields = field
        .attrs
        .iter()
        .filter(|attr| {
            if let Some(ident) = attr.path().get_ident() {
                RECOGNIZED_FIELDS.contains(&ident.to_string().as_str())
            } else {
                false
            }
        })
        .collect::<Vec<_>>();
    if recognized_fields.len() != 1 {
        return Err(syn::Error::new_spanned(
            &field,
            format!(
                "Exactly one of #[track], #[check], #[color], #[select], or #[file] is required (found {})",
                recognized_fields.len()
            ),
        ));
    }
    let recognized_attr = recognized_fields[0];
    match recognized_attr
        .path()
        .get_ident()
        .unwrap()
        .to_string()
        .as_str()
    {
        "track" => filter_config_field_track(field, recognized_attr),
        "check" => filter_config_field_check(field, recognized_attr),
        "color" => filter_config_field_color(field, recognized_attr),
        "select" => filter_config_field_select(field, recognized_attr),
        "file" => filter_config_field_file(field, recognized_attr),

        _ => unreachable!(),
    }
}

fn filter_config_field_track(
    field: &syn::Field,
    recognized_attr: &syn::Attribute,
) -> Result<FilterConfigField, syn::Error> {
    let mut name = None;
    let mut default = None;
    let mut min = None;
    let mut max = None;
    let mut step = None;

    recognized_attr.parse_nested_meta(|m| {
        if m.path.is_ident("name") {
            name = Some(m.value()?.parse::<syn::LitStr>()?.value());
        } else if m.path.is_ident("step") {
            let value_token = m.value()?.parse::<syn::LitFloat>()?;
            let value = value_token.base10_parse::<TrackStep>()?;

            step = Some(value);
        } else if m.path.is_ident("range") {
            let value_token = m.value()?;
            let expr = value_token.parse::<syn::Expr>()?;
            if let syn::Expr::Range(expr_range) = expr {
                if !matches!(expr_range.limits, syn::RangeLimits::Closed(_)) {
                    return Err(syn::Error::new_spanned(
                        expr_range,
                        "range must be a closed range (e.g., 0.0..=1.0)",
                    ));
                }
                if let Some(ref from) = expr_range.start {
                    min = Some(parse_int_or_float(from)?);
                } else {
                    return Err(syn::Error::new_spanned(
                        expr_range,
                        "range must have a start value",
                    ));
                }
                if let Some(to) = expr_range.end {
                    max = Some(parse_int_or_float(&to)?);
                } else {
                    return Err(syn::Error::new_spanned(
                        expr_range,
                        "range must have an end value",
                    ));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    expr,
                    "range must be a range expression (e.g., 0.0..=1.0)",
                ));
            }
        } else if m.path.is_ident("default") {
            let value_token = m.value()?;
            default = Some(parse_int_or_float(&value_token.parse()?)?);
        } else {
            return Err(m.error("Unknown attribute for track"));
        }
        Ok(())
    })?;

    let Some(step) = step else {
        return Err(syn::Error::new_spanned(recognized_attr, "step is required"));
    };

    let name = name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
    let (Some(default), Some(min), Some(max)) = (default, min, max) else {
        return Err(syn::Error::new_spanned(
            recognized_attr,
            "default, min, and max are required",
        ));
    };
    if !(min <= default && default <= max) {
        return Err(syn::Error::new_spanned(
            recognized_attr,
            "default must be between min and max",
        ));
    }
    let step_value = decimal_rs::Decimal::from(step);
    match (min % step_value, max % step_value, default % step_value) {
        (d, _, _) if d != decimal_rs::Decimal::ZERO => {
            return Err(syn::Error::new_spanned(
                recognized_attr,
                "min must be a multiple of step",
            ));
        }
        (_, d, _) if d != decimal_rs::Decimal::ZERO => {
            return Err(syn::Error::new_spanned(
                recognized_attr,
                "max must be a multiple of step",
            ));
        }
        (_, _, d) if d != decimal_rs::Decimal::ZERO => {
            return Err(syn::Error::new_spanned(
                recognized_attr,
                "default must be a multiple of step",
            ));
        }
        _ => {}
    }
    Ok(FilterConfigField::Track {
        id: field.ident.as_ref().unwrap().to_string(),
        name,
        default: default.into(),
        min: min.into(),
        max: max.into(),
        step,
    })
}

fn filter_config_field_check(
    field: &syn::Field,
    recognized_attr: &syn::Attribute,
) -> Result<FilterConfigField, syn::Error> {
    let mut name = None;
    let mut default = None;

    recognized_attr.parse_nested_meta(|m| {
        if m.path.is_ident("name") {
            name = Some(m.value()?.parse::<syn::LitStr>()?.value());
        } else if m.path.is_ident("default") {
            default = Some(m.value()?.parse::<syn::LitBool>()?.value);
        } else {
            return Err(m.error("Unknown attribute for check"));
        }
        Ok(())
    })?;

    let name = name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
    let Some(default) = default else {
        return Err(syn::Error::new_spanned(
            recognized_attr,
            "default is required",
        ));
    };
    Ok(FilterConfigField::Check {
        id: field.ident.as_ref().unwrap().to_string(),
        name,
        default,
    })
}

fn filter_config_field_color(
    field: &syn::Field,
    recognized_attr: &syn::Attribute,
) -> Result<FilterConfigField, syn::Error> {
    let mut name = None;
    let mut default = None;

    recognized_attr.parse_nested_meta(|m| {
        if m.path.is_ident("name") {
            name = Some(m.value()?.parse::<syn::LitStr>()?.value());
        } else if m.path.is_ident("default") {
            let lit = m.value()?;
            default = Some(
                lit.parse::<syn::Lit>()
                    .and_then(|lit| parse_color_lit(&lit))
                    .or_else(|_| {
                        let expr = lit.parse::<syn::Expr>()?;
                        if let syn::Expr::Tuple(expr_tuple) = expr {
                            parse_color_tuple(&expr_tuple)
                        } else {
                            Err(syn::Error::new_spanned(
                                expr,
                                "Failed to parse color (expected integer, string literal, or tuple)",
                            ))
                        }
                    })?,
                );
        } else {
            return Err(m.error("Unknown attribute for color"));
        }

        Ok(())

    })?;

    let name = name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
    let Some(default) = default else {
        return Err(syn::Error::new_spanned(
            recognized_attr,
            "default is required",
        ));
    };
    return Ok(FilterConfigField::Color {
        id: field.ident.as_ref().unwrap().to_string(),
        name,
        default,
    });

    fn parse_color_lit(lit: &syn::Lit) -> Result<u32, syn::Error> {
        match lit {
            syn::Lit::Int(lit_int) => {
                let value = lit_int.base10_parse::<u32>()?;
                if value > 0xFFFFFF {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "Color value must be between 0x000000 and 0xFFFFFF",
                    ));
                }
                Ok(value)
            }
            syn::Lit::Str(lit_str) => {
                let s = lit_str.value();
                let s = s.trim_start_matches('#');
                if s.len() != 6 {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "Color string must be in the format #RRGGBB",
                    ));
                }
                let value = u32::from_str_radix(s, 16).map_err(|_| {
                    syn::Error::new_spanned(lit, "Color string must be in the format #RRGGBB")
                })?;
                Ok(value)
            }
            _ => Err(syn::Error::new_spanned(
                lit,
                "Failed to parse color (expected integer or string literal)",
            )),
        }
    }
    fn parse_color_tuple(lit: &syn::ExprTuple) -> Result<u32, syn::Error> {
        if lit.elems.len() != 3 {
            return Err(syn::Error::new_spanned(
                lit,
                "Color tuple must have exactly 3 elements",
            ));
        }
        let mut rgb = [0u8; 3];
        for (i, expr) in lit.elems.iter().enumerate() {
            match expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(lit_int),
                    ..
                }) => {
                    let value = lit_int.base10_parse::<u8>()?;
                    rgb[i] = value;
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "Color tuple elements must be integer literals",
                    ));
                }
            }
        }
        Ok(((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32))
    }
}

fn filter_config_field_select(
    field: &syn::Field,
    recognized_attr: &syn::Attribute,
) -> Result<FilterConfigField, syn::Error> {
    let mut name = None;
    let mut default = None;
    let mut items = None;

    recognized_attr.parse_nested_meta(|m| {
        if m.path.is_ident("name") {
            name = Some(m.value()?.parse::<syn::LitStr>()?.value());
        } else if m.path.is_ident("default") {
            let value_token = m.value()?.parse::<syn::LitInt>()?;
            let value = value_token.base10_parse::<i32>()?;
            default = Some(value);
        } else if m.path.is_ident("items") {
            let value_token = m.value()?;
            let expr = value_token.parse::<syn::Expr>()?;
            if let syn::Expr::Array(expr_array) = expr {
                let mut opts = Vec::new();
                for elem in expr_array.elems.iter() {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = elem
                    {
                        opts.push(lit_str.value());
                    } else {
                        return Err(syn::Error::new_spanned(
                            elem,
                            "Options must be string literals",
                        ));
                    }
                }
                items = Some(opts);
            } else {
                return Err(syn::Error::new_spanned(
                    expr,
                    "Options must be an array of string literals",
                ));
            }
        } else {
            return Err(m.error("Unknown attribute for select"));
        }
        Ok(())
    })?;

    let name = name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
    let (Some(default), Some(items)) = (default, items) else {
        return Err(syn::Error::new_spanned(
            recognized_attr,
            "default and items are required",
        ));
    };
    if !(0 <= default && (default as usize) < items.len()) {
        return Err(syn::Error::new_spanned(
            recognized_attr,
            "default must be a valid index into items",
        ));
    }
    Ok(FilterConfigField::Select {
        id: field.ident.as_ref().unwrap().to_string(),
        name,
        default,
        items,
    })
}

fn filter_config_field_file(
    field: &syn::Field,
    recognized_attr: &syn::Attribute,
) -> Result<FilterConfigField, syn::Error> {
    let mut name = None;
    let mut filter = None;

    recognized_attr.parse_nested_meta(|m| {
        if m.path.is_ident("name") {
            name = Some(m.value()?.parse::<syn::LitStr>()?.value());
        } else if m.path.is_ident("filters") {
            let content;
            syn::braced!(content in &m.value()?);
            filter = Some(
                content
                    .parse_terminated(FileFilterEntry::parse, syn::Token![,])?
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
        } else {
            return Err(m.error("Unknown attribute for file"));
        }
        Ok(())
    })?;

    let name = name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
    let Some(filter) = filter else {
        return Err(syn::Error::new_spanned(
            recognized_attr,
            "filters is required",
        ));
    };
    Ok(FilterConfigField::File {
        id: field.ident.as_ref().unwrap().to_string(),
        name,
        filters: filter,
    })
}

fn parse_int_or_float(expr: &syn::Expr) -> Result<decimal_rs::Decimal, syn::Error> {
    let mut current = expr;
    let mut neg_count = 0;
    // Iteratively handle nested unary negations
    loop {
        match current {
            syn::Expr::Unary(syn::ExprUnary {
                op: syn::UnOp::Neg(_),
                expr,
                ..
            }) => {
                neg_count += 1;
                current = &**expr;
            }
            syn::Expr::Paren(syn::ExprParen { expr, .. }) => {
                current = &**expr;
            }
            _ => break,
        }
    }
    match current {
        syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
            syn::Lit::Int(lit_int) => {
                let v = lit_int.base10_parse::<decimal_rs::Decimal>()?;
                if neg_count % 2 == 0 { Ok(v) } else { Ok(-v) }
            }
            syn::Lit::Float(lit_float) => {
                let v = lit_float.base10_parse::<decimal_rs::Decimal>()?;
                if neg_count % 2 == 0 { Ok(v) } else { Ok(-v) }
            }
            _ => Err(syn::Error::new_spanned(
                lit,
                "Expected integer or float literal",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            current,
            "Expected integer or float literal",
        )),
    }
}
