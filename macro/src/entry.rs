use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Error, Field};

use crate::attribute::EntryAttributes;

pub(crate) fn gen_entry_ensure(field: &Field) -> TokenStream {
    let ty = &field.ty;
    quote! {
        const _: fn() = || {
            fn assert_impl<T: UnitEntry>() {}
            assert_impl::<#ty>();
        };
    }
}

pub(crate) fn gen_entry_parse(field: &Field) -> Result<TokenStream, Error> {
    let name = &field
        .ident
        .as_ref()
        .expect("Tuple structs are not supported.");
    let ty = &field.ty;
    let attributes = EntryAttributes::parse_vec(&field.attrs)?;
    let key = attributes.key.unwrap_or((&field.ident).into_token_stream());

    let key_name = format!("{}", key);

    let result = match attributes.default {
        Some(default) => {
            quote! {
                let #name = #ty::parse(__source, #key_name).unwrap_or(#default);
            }
        }
        None => {
            quote! {
                let #name = #ty::parse(__source, #key_name).ok_or(systemd_unit_parser::error::EntryMissingError { key: #key_name })?;
            }
        }
    };

    Ok(result)
}

// pub(crate) fn gen_entry_derives(input: DeriveInput) -> syn::Result<TokenStream> {
//     todo!()
// }
