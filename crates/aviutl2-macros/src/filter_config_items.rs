use quote::ToTokens;
use syn::parse::Parse;

pub fn filter_config_items(
    item: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let item: syn::ItemStruct = syn::parse2(item).map_err(|e| e.to_compile_error())?;
    validate_filter_config(&item)?;

    let name = &item.ident;
    let fields = item
        .fields
        .iter()
        .map(filter_config_field)
        .collect::<crate::utils::CombinedVecResults<_>>()
        .into_result()?;
    let to_config_items = impl_to_config_items(&fields);
    let from_config_items = impl_from_filter_config(&fields);
    let default = impl_default(&fields);

    let expanded = quote::quote! {
        #[automatically_derived]
        impl ::aviutl2::filter::FilterConfigItems for #name {
            #to_config_items

            #from_config_items
        }

        #[automatically_derived]
        impl ::std::default::Default for #name {
            fn default() -> Self {
                #default
            }
        }
    };

    Ok(expanded)
}

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
        default: either::Either<i32, syn::ExprPath>,
        items: either::Either<Vec<String>, syn::TypePath>,
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

fn impl_to_config_items(fields: &[FilterConfigField]) -> proc_macro2::TokenStream {
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
                    }
                    TrackStep::PointOne => {
                        quote::quote! { ::aviutl2::filter::FilterConfigTrackStep::PointOne }
                    }
                    TrackStep::PointZeroOne => {
                        quote::quote! { ::aviutl2::filter::FilterConfigTrackStep::PointZeroOne }
                    }
                    TrackStep::PointZeroZeroOne => {
                        quote::quote! { ::aviutl2::filter::FilterConfigTrackStep::PointZeroZeroOne }
                    }
                };
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Track(
                        ::aviutl2::filter::FilterConfigTrack {
                            name: #name.to_string(),
                            value: #default,
                            range: #min..=#max,
                            step: #step_enum,
                        }
                    )
                }
            }
            FilterConfigField::Check {
                id: _,
                name,
                default,
            } => {
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Checkbox(
                        ::aviutl2::filter::FilterConfigCheckbox {
                            name: #name.to_string(),
                            value: #default,
                        }
                    )
                }
            }
            FilterConfigField::Color {
                id: _,
                name,
                default,
            } => {
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Color(
                        ::aviutl2::filter::FilterConfigColor {
                            name: #name.to_string(),
                            value: #default.into(),
                        }
                    )
                }
            }
            FilterConfigField::Select {
                id: _,
                name,
                default,
                items,
            } => {
                let items = match items {
                    either::Either::Left(items) => {
                        let items = items.iter().enumerate().map(|(i, item)| {
                            quote::quote! {
                                ::aviutl2::filter::FilterConfigSelectItem {
                                    name: #item.to_string(),
                                    value: #i as i32,
                                }
                            }
                        });
                        quote::quote! { vec![#(#items),*] }
                    }
                    either::Either::Right(ty) => {
                        quote::quote! { <#ty as ::aviutl2::filter::FilterConfigSelectItems>::to_select_items() }
                    }
                };
                let default = match default {
                    either::Either::Left(v) => quote::quote! { #v },
                    either::Either::Right(v) => quote::quote! { ::aviutl2::filter::FilterConfigSelectItems::to_select_item_value(&#v) },
                };
                quote::quote! {
                    ::aviutl2::filter::FilterConfigItem::Select(
                        ::aviutl2::filter::FilterConfigSelect {
                            name: #name.to_string(),
                            value: #default,
                            items: #items,
                        }
                    )
                }
            }
            FilterConfigField::File {
                id: _,
                name,
                filters: filter,
            } => {
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
                    ::aviutl2::filter::FilterConfigItem::File(
                        ::aviutl2::filter::FilterConfigFile {
                            name: #name.to_string(),
                            value: String::new(),
                            filters: vec![#(#filter_entries),*],
                        }
                    )
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

fn impl_from_filter_config(config_fields: &[FilterConfigField]) -> proc_macro2::TokenStream {
    let field_assign = config_fields
        .iter()
        .enumerate()
        .map(|(i, f)| match f {
            FilterConfigField::Track { id, step, .. } => {
                let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
                let to_value = if *step == TrackStep::One {
                    // 一回i32に変換する
                    quote::quote! {
                         (track.value as i32) as _
                    }
                } else {
                    quote::quote! {
                        track.value as _
                    }
                };
                quote::quote! {
                    #id_ident: match items[#i] {
                        ::aviutl2::filter::FilterConfigItem::Track(ref track) => #to_value,
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
                        ::aviutl2::filter::FilterConfigItem::Color(ref color) => color.value.into(),
                        _ => panic!("Expected Color at index {}", #i),
                    }
                }
            }
            FilterConfigField::Select {
                id, items, default, ..
            } => {
                // defaultが：
                //   i32（Left）：インデックスで返す
                //   syn::TypePath（Right）：FilterConfigSelectItems::from_select_item_valueで変換して返す
                let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
                let to_value = match default {
                    either::Either::Left(_) => {
                        quote::quote! {
                            (select.value as usize) as _
                        }
                    }
                    either::Either::Right(_) => match items {
                        either::Either::Left(items) => {
                            quote::quote! {
                                [#(#items),*][select.value as usize].into()
                            }
                        }
                        either::Either::Right(type_path) => {
                            let type_path = type_path.to_token_stream();
                            quote::quote! {
                                <#type_path as ::aviutl2::filter::FilterConfigSelectItems>::from_select_item_value(select.value)
                            }
                        }
                    },
                };

                quote::quote! {
                    #id_ident: match items[#i] {
                        ::aviutl2::filter::FilterConfigItem::Select(ref select) => {
                            #to_value
                        },
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

fn impl_default(fields: &[FilterConfigField]) -> proc_macro2::TokenStream {
    let field_inits = fields.iter().map(|f| match f {
        FilterConfigField::Track { id, default, .. } => {
            let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
            quote::quote! {
                #id_ident: #default as _
            }
        }
        FilterConfigField::Check { id, default, .. } => {
            let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
            quote::quote! {
                #id_ident: #default
            }
        }
        FilterConfigField::Color { id, default, .. } => {
            let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
            quote::quote! {
                #id_ident: #default.into()
            }
        }
        FilterConfigField::Select { id, default, .. } => {
            let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
            match default {
                either::Either::Left(v) => quote::quote! {
                    #id_ident: #v as _
                },
                either::Either::Right(v) => quote::quote! {
                    #id_ident: <_ as ::std::convert::From<_>>::from(#v)
                },
            }
        }
        FilterConfigField::File { id, .. } => {
            let id_ident = syn::Ident::new(id, proc_macro2::Span::call_site());
            quote::quote! {
                #id_ident: None
            }
        }
    });
    quote::quote! {
        Self {
            #(#field_inits),*
        }
    }
}

fn validate_filter_config(item: &syn::ItemStruct) -> Result<(), proc_macro2::TokenStream> {
    let fields = item
        .fields
        .iter()
        .map(filter_config_field)
        .collect::<Result<Vec<_>, _>>();
    let fields = match fields {
        Ok(f) => f,
        Err(e) => return Err(e.to_compile_error()),
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
        return Err(syn::Error::new_spanned(item, "Field names must be unique").to_compile_error());
    }
    Ok(())
}

fn filter_config_field(field: &syn::Field) -> Result<FilterConfigField, syn::Error> {
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
            field,
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
            let value = m.value()?;
            let lookahead = value.lookahead1();
            if lookahead.peek(syn::LitInt) {
                let lit = value.parse::<syn::LitInt>()?;
                let v = lit.base10_parse::<i32>()?;
                default = Some(either::Either::Left(v));
            } else if lookahead.peek(syn::Ident) || lookahead.peek(syn::Token![::]) {
                let expr = value.parse::<syn::Expr>()?;
                if let syn::Expr::Path(expr) = expr {
                    default = Some(either::Either::Right(expr.clone()));
                } else {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "default must be an integer literal or a path expression",
                    ));
                }
            } else {
                return Err(lookahead.error());
            }
        } else if m.path.is_ident("items") {
            let value_token = m.value()?;
            let lookahead = value_token.lookahead1();
            if lookahead.peek(syn::token::Bracket) {
                let expr_array = value_token.parse::<syn::ExprArray>()?;
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
                items = Some(either::Either::Left(opts));
            } else if lookahead.peek(syn::Ident) || lookahead.peek(syn::Token![::]) {
                items = Some(either::Either::Right(value_token.parse::<syn::TypePath>()?));
            } else {
                return Err(lookahead.error());
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

    if let (either::Either::Left(items), either::Either::Left(&default)) =
        (items.as_ref(), default.as_ref())
        && !(0 <= default && (default as usize) < items.len())
    {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track() {
        let input: proc_macro2::TokenStream = quote::quote! {
            struct Config {
                #[track(name = "Frequency", range = 20.0..=20000.0, step = 1.0, default = 440.0)]
                frequency: f64,
            }
        };
        let output = filter_config_items(input).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }

    #[test]
    fn test_check() {
        let input: proc_macro2::TokenStream = quote::quote! {
            struct Config {
                #[check(name = "Enable", default = true)]
                enable: bool,
            }
        };
        let output = filter_config_items(input).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }

    #[test]
    fn test_color() {
        let input: proc_macro2::TokenStream = quote::quote! {
            struct Config {
                #[color(name = "IntColor", default = 0xFF00FF)]
                int_color: u32,
                #[color(name = "StrColor", default = "#00FF00")]
                str_color: u32,
                #[color(name = "TupleColor", default = (255, 0, 0))]
                tuple_color: u32,
            }
        };
        let output = filter_config_items(input).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }

    #[test]
    fn test_select() {
        let input: proc_macro2::TokenStream = quote::quote! {
            struct Config {
                #[select(name = "Mode", items = ["Easy", "Medium", "Hard"], default = 1)]
                mode: usize,
            }
        };
        let output = filter_config_items(input).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }

    #[test]
    #[allow(dead_code)]
    fn test_select_behavior() {
        use aviutl2::filter::FilterConfigItems;

        #[derive(Debug, PartialEq, Eq, aviutl2::filter::FilterConfigSelectItems)]
        enum Behavior {
            #[item(name = "Easy")]
            Easy,
            #[item(name = "Medium")]
            Medium,
            #[item(name = "Hard")]
            Hard,
        }

        #[derive(aviutl2::filter::FilterConfigItems)]
        struct Config {
            #[select(name = "Mode", items = ["Easy", "Medium", "Hard"], default = 1)]
            mode: usize,

            #[select(name = "Behavior 1", items = Behavior, default = Behavior::Medium)]
            behavior1: Behavior,

            #[select(name = "Behavior 2", items = Behavior, default = 1)]
            behavior2: usize,
        }

        let items = Config::to_config_items();
        insta::assert_debug_snapshot!(items);
    }

    #[test]
    fn test_file() {
        let input: proc_macro2::TokenStream = quote::quote! {
            struct Config {
                #[file(name = "Input File", filters = { "Text Files" => ["*.txt"], "All Files" => ["*.*"] })]
                input_file: Option<std::path::PathBuf>,
            }
        };
        let output = filter_config_items(input).unwrap();
        insta::assert_snapshot!(rustfmt_wrapper::rustfmt(output).unwrap());
    }

    #[test]
    fn test_duplicate_field_name() {
        let input: proc_macro2::TokenStream = quote::quote! {
            struct Config {
                #[track(name = "Frequency", range = 20.0..=20000.0, step = 1.0, default = 440.0)]
                frequency1: f64,
                #[track(name = "Frequency", range = 20.0..=20000.0, step = 1.0, default = 440.0)]
                frequency2: f64,
            }
        };
        let result = filter_config_items(input);
        assert!(result.is_err());
    }
}
