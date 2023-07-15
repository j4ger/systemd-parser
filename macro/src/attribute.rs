use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, LitStr, Meta};

pub(crate) fn impl_default(input: &Vec<Attribute>) -> syn::Result<bool> {
    for attribute in input.iter() {
        if attribute.path().is_ident("unit") {
            let mut result = false;
            attribute.parse_nested_meta(|nested| {
                Ok(if nested.path.is_ident("default") {
                    result = true;
                })
            })?;
            if result {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub(crate) fn impl_key(input: &Vec<Attribute>) -> syn::Result<Option<TokenStream>> {
    for attribute in input.iter() {
        if attribute.path().is_ident("unit") {
            let mut result = None;
            attribute.parse_nested_meta(|nested| {
                if nested.path.is_ident("key") {
                    let inner = nested.value()?;
                    let lit: LitStr = inner.parse()?;
                    result = Some(lit.into_token_stream());
                }
                Ok(())
            })?;
            if result.is_some() {
                return Ok(result);
            }
        }
    }
    Ok(None)
}

pub(crate) fn impl_default_val(input: &Vec<Attribute>) -> syn::Result<Option<TokenStream>> {
    for attribute in input.iter() {
        if attribute.path().is_ident("unit") {
            let mut result = None;
            attribute.parse_nested_meta(|nested| {
                if nested.path.is_ident("default") {
                    let inner = nested.value()?;
                    let lit: LitStr = inner.parse()?;
                    result = Some(lit.into_token_stream());
                }
                Ok(())
            })?;
            if result.is_some() {
                return Ok(result);
            }
        }
    }
    Ok(None)
}
