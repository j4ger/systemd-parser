use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, Lit, Type};

pub(crate) fn transform_default(ty: &Type, default: &Expr) -> TokenStream {
    // add `to_string()` suffix if ty is String
    if let Type::Path(inner) = ty {
        let path = inner.path.segments.last().expect("Invalid type.");
        if path.ident == "String" {
            if let Expr::Lit(expr) = default {
                if let Lit::Str(string) = &expr.lit {
                    return format!("\"{}\".to_string()", string.value())
                        .parse()
                        .expect("Invalid default value.");
                }
            }
        }
    }
    return default.into_token_stream();
}
