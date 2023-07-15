use crate::attribute::{impl_default_val, impl_key};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    token::Brace,
    Attribute, DeriveInput, Error, Expr, Field, Ident, Meta, Token, TypePath, Visibility,
};

pub(crate) fn gen_entry_ensure(field: &Field) -> TokenStream {
    let ty = &field.ty;
    quote! {
        const _: fn() = || {
            fn assert_impl<T: UnitEntry>() {}
            assert_impl::<ty>();
        };
    }
}

pub(crate) fn gen_entry_parse(field: &Field) -> Result<TokenStream, Error> {
    let name = &field.ident;
    let ident = format_ident!(
        "__unit_{}",
        name.as_ref().expect("Tuple structs are not supported.")
    );
    let ty = &field.ty;
    let key = impl_key(&field.attrs)?.unwrap_or((&field.ident).into_token_stream());
    let default = impl_default_val(&field.attrs)?;

    let key_name = format!("{}", key);

    let result = match default {
        Some(default) => {
            quote! {
                let #ident = ty::parse(source)?.unwrap_or(default);
            }
        }
        None => {
            quote! {
                let #ident = ty::parse(source)?.ok_or(systemd_unit_parser::error::EntryMissingError { key: #key_name })?;
            }
        }
    };

    Ok(result)
}
