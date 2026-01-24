use quote::ToTokens;

pub fn macro_if(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let mut tokens = input.into_iter();
    syn::parse2::<syn::Token![if]>(tokens.next().into_token_stream())
        .map_err(|e| e.into_compile_error())?;
    let condition: syn::Expr =
        syn::parse2(tokens.next().into_token_stream()).map_err(|e| e.into_compile_error())?;
    syn::parse2::<syn::Token![in]>(tokens.next().into_token_stream())
        .map_err(|e| e.into_compile_error())?;

    let variables = match tokens.next() {
        Some(proc_macro2::TokenTree::Group(group))
            if group.delimiter() == proc_macro2::Delimiter::Parenthesis =>
        {
            let inner_tokens = group.stream();
            parse_kv_variables(inner_tokens.clone())?
        }
        _ => {
            return Err(syn::Error::new_spanned(
                condition,
                "expected a parenthesized name-value pair",
            )
            .into_compile_error());
        }
    };

    let then_block = syn::parse2::<syn::Block>(tokens.next().into_token_stream())
        .map_err(|e| e.into_compile_error())?;
    let maybe_else_token = tokens.next();
    let else_block = if let Some(token) = maybe_else_token {
        syn::parse2::<syn::Token![else]>(token.into_token_stream())
            .map_err(|e| e.into_compile_error())?;
        Some(
            syn::parse2::<syn::Block>(tokens.next().into_token_stream())
                .map_err(|e| e.into_compile_error())?,
        )
    } else {
        None
    };

    let condition_value = evaluate_condition(&condition, &variables)?;
    let mut output = proc_macro2::TokenStream::new();
    if condition_value {
        output.extend(then_block.stmts.iter().map(|s| s.to_token_stream()));
    } else if let Some(else_block) = else_block {
        output.extend(else_block.stmts.iter().map(|s| s.to_token_stream()));
    }
    Ok(output)
}

fn parse_kv_variables(
    inner_tokens: proc_macro2::TokenStream,
) -> Result<std::collections::HashMap<String, bool>, proc_macro2::TokenStream> {
    let mut inner_iter = inner_tokens.into_iter();
    let mut variables = std::collections::HashMap::new();
    loop {
        let maybe_next = inner_iter.next();
        let Some(maybe_next) = maybe_next else { break };
        let name: syn::Ident =
            syn::parse2(maybe_next.to_token_stream()).map_err(|e| e.into_compile_error())?;
        syn::parse2::<syn::Token![=]>(inner_iter.next().into_token_stream())
            .map_err(|e| e.into_compile_error())?;
        let value: syn::LitBool = syn::parse2(inner_iter.next().into_token_stream())
            .map_err(|e| e.into_compile_error())?;
        variables.insert(name.to_string(), value.value);
        if let Some(next_token) = inner_iter.next() {
            syn::parse2::<syn::Token![,]>(next_token.into_token_stream())
                .map_err(|e| e.into_compile_error())?;
        } else {
            break;
        }
    }
    Ok(variables)
}

fn evaluate_condition(
    condition: &syn::Expr,
    variables: &std::collections::HashMap<String, bool>,
) -> Result<bool, proc_macro2::TokenStream> {
    match condition {
        syn::Expr::Paren(expr_paren) => evaluate_condition(&expr_paren.expr, variables),
        syn::Expr::Binary(expr_binary) => {
            let left = evaluate_condition(&expr_binary.left, variables)?;
            let right = evaluate_condition(&expr_binary.right, variables)?;
            match expr_binary.op {
                syn::BinOp::And(_) => Ok(left && right),
                syn::BinOp::Or(_) => Ok(left || right),
                syn::BinOp::Eq(_) => Ok(left == right),
                syn::BinOp::Ne(_) => Ok(left != right),
                syn::BinOp::BitXor(_) => Ok(left ^ right),
                syn::BinOp::BitAnd(_) => Ok(left & right),
                syn::BinOp::BitOr(_) => Ok(left | right),
                _ => Err(
                    syn::Error::new_spanned(condition, "unsupported binary operator")
                        .into_compile_error(),
                ),
            }
        }
        syn::Expr::Unary(expr_unary) => {
            let expr_value = evaluate_condition(&expr_unary.expr, variables)?;
            match expr_unary.op {
                syn::UnOp::Not(_) => Ok(!expr_value),
                _ => Err(
                    syn::Error::new_spanned(condition, "unsupported unary operator")
                        .into_compile_error(),
                ),
            }
        }
        syn::Expr::Path(expr_path) => {
            if expr_path.path.segments.len() == 1 {
                let var_name = expr_path.path.segments[0].ident.to_string();
                if let Some(value) = variables.get(&var_name) {
                    Ok(*value)
                } else {
                    Ok(false) // Default to false if variable not found
                }
            } else {
                Err(
                    syn::Error::new_spanned(condition, "unsupported path expression")
                        .into_compile_error(),
                )
            }
        }
        _ => Err(syn::Error::new_spanned(condition, "unsupported expression").into_compile_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    #[test]
    fn test_simple_expr() {
        let input: TokenStream = quote! {
            if hoge in (hoge = true) {
                println!("hoge is true");
            } else {
                println!("hoge is false");
            }
        };
        let output = macro_if(input).unwrap();
        let expected: TokenStream = quote! {
            println!("hoge is true");
        };
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_duplicated_variables() {
        let input: TokenStream = quote! {
            if hoge in (hoge = true, hoge = false) {
                println!("hoge is true");
            } else {
                println!("hoge is false");
            }
        };
        let output = macro_if(input).unwrap();
        let expected: TokenStream = quote! {
            println!("hoge is false");
        };
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_complex_expr() {
        let input: TokenStream = quote! {
            if ((hoge & fuga) | !piyo) in (hoge = true, fuga = false, piyo = false) {
                println!("condition is true");
            } else {
                println!("condition is false");
            }
        };
        let output = macro_if(input).unwrap();
        let expected: TokenStream = quote! {
            println!("condition is true");
        };
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_no_else() {
        let input: TokenStream = quote! {
            if hoge in (hoge = true) {
                println!("hoge is true");
            }
        };
        let output = macro_if(input).unwrap();
        let expected: TokenStream = quote! {
            println!("hoge is true");
        };
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_no_else_empty() {
        let input: TokenStream = quote! {
            if hoge in (hoge = false) {
                println!("hoge is true");
            }
        };
        let output = macro_if(input).unwrap();
        assert!(output.is_empty());
    }
}
