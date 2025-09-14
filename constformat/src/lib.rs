#![allow(non_snake_case)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    Expr, ExprLit, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct FormatInput {
    fmt: LitStr,
    args: Vec<Expr>,
}

impl Parse for FormatInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fmt = input.parse()?;
        let mut args = Vec::new();
        while input.parse::<Token![,]>().is_ok() {
            args.push(input.parse()?);
        }

        Ok(FormatInput { fmt, args })
    }
}

#[proc_macro]
pub fn const_format(input: TokenStream) -> TokenStream {
    let FormatInput { fmt: fmtExpr, args } = parse_macro_input!(input as FormatInput);

    //let fmtVal = evaluateFormatExpr(&fmtExpr).unwrap_or_else(|err| err.to_compile_error().to_string());
    let fmtVal = fmtExpr.value();

    let splits: Vec<&str> = fmtVal.split("{}").collect();
    if splits.len() != args.len() + 1 {
        return syn::Error::new(
            fmtExpr.span(),
            "Number of arguments does not match the number of placeholders",
        )
        .to_compile_error()
        .into();
    }

    let mut output = String::new();
    for (i, split) in splits.iter().enumerate() {
        output.push_str(split);
        if i < args.len() {
            let arg = &args[i];
            if let Expr::Lit(litExpr) = arg {
                if let Lit::Verbatim(_) = &litExpr.lit {
                    return syn::Error::new(arg.span(), "Unsupported literal type")
                        .to_compile_error()
                        .into();
                }

                output.push_str(LitToStr(litExpr).as_str());
            } else if let Expr::Macro(macExpr) = arg {
                let macroName = macExpr.mac.path.segments.last().unwrap().ident.to_string();
                match macroName.as_str() {
                    "file" => {
                        let file = arg
                            .span()
                            .source_file()
                            .path()
                            .to_string_lossy()
                            .to_string();
                        output.push_str(&file);
                    }

                    "line" => {
                        let line = arg.span().start().line;
                        output.push_str(&line.to_string());
                    }

                    "column" => {
                        // 0-indexed, add 1 to make human-readable
                        let column = Span::call_site().start().column + 1;
                        output.push_str(&column.to_string());
                    }

                    "concat" => {
                        let concatArgs = macExpr.mac.tokens.to_string();
                        let concatArgs = concatArgs.split(",").collect::<Vec<&str>>();
                        let mut concatOutput = String::new();
                        for arg in concatArgs {
                            if let Ok(lit) = syn::parse_str::<LitStr>(arg) {
                                concatOutput.push_str(&lit.value());
                            } else {
                                return syn::Error::new(arg.span(), "Argument must be a literal")
                                    .to_compile_error()
                                    .into();
                            }
                        }
                        output.push_str(&concatOutput);
                    }

                    _ => {
                        return syn::Error::new(
                            arg.span(),
                            "Unsupported macro \"".to_string() + macroName.as_str() + "\"",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            } else {
                return syn::Error::new(arg.span(), "Argument must be a literal")
                    .to_compile_error()
                    .into();
            }
        }
    }

    let result = LitStr::new(&output, fmtExpr.span());
    let tokens = quote! {
        #result
    };

    tokens.into()
}

fn LitToStr(exprLit: &ExprLit) -> String {
    match &exprLit.lit {
        Lit::Str(litStr) => litStr.value(),

        Lit::Int(litInt) => litInt.base10_digits().to_string(),

        Lit::Float(litFloat) => litFloat.base10_digits().to_string(),

        Lit::Bool(litBool) => {
            if litBool.value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }

        Lit::Char(litChar) => litChar.value().to_string(),

        Lit::ByteStr(litByteStr) => String::from_utf8_lossy(&litByteStr.value()).to_string(),

        Lit::CStr(litCStr) => {
            String::from_utf8_lossy(&litCStr.value().as_bytes_with_nul()).to_string()
        }

        Lit::Byte(litByte) => litByte.value().to_string(),
        _ => {
            // should never be reached
            String::new()
        }
    }
}

/*
fn evaluateFormatExpr(expr: &Expr) -> Result<String, syn::Error> {
    if let Expr::Lit(exprLit) = expr {
        if let Lit::Str(strLit) = &exprLit.lit {
            return Ok(strLit.value());
        }
        return Err(syn::Error::new(expr.span(), "Expected a string literal"));

    } else if let Expr::Macro(macExpr) = expr {
        let macIdent = macExpr.mac.path.segments.last().unwrap().ident.to_string();
        if macIdent == "concat" {
            let tokens = macExpr.mac.tokens.clone();
            let parsed: syn::punctuated::Punctuated<Expr, Token![,]> = syn::parse2(tokens)?;
            let mut result = String::new();
            for e in parsed {
                if let Expr::Lit(lit_expr) = e {
                    if let Lit::Str(lit_str) = lit_expr.lit {
                        result.push_str(&lit_str.value());
                    } else {
                        return Err(syn::Error::new(e.span(), "concat! only supports string literals"));
                    }
                } else {
                    return Err(syn::Error::new(e.span(), "concat! only supports string literals"));
                }
            }
            return Ok(result);
        }
        return Err(syn::Error::new(expr.span(), "Expected a string literal or concat!(...)"));
    }
    Err(syn::Error::new(expr.span(), "Expected a string literal or concat!(...)"))
}
*/
