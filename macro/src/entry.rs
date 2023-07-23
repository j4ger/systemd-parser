use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Error, Field};

use crate::{attribute::EntryAttributes, transform_default::transform_default};

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
    let name = field
        .ident
        .as_ref()
        .expect("Tuple structs are not supported.");
    let ty = &field.ty;
    let attributes = EntryAttributes::parse_vec(&field.attrs)?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match attributes.default {
        Some(default) => {
            let default = transform_default(ty, &default);
            quote! {
                let #name: #ty = systemd_parser::internal::UnitEntry::__parse_entry(__source, #key)?.unwrap_or(#default);
            }
        }
        None => {
            quote! {
                let #name: #ty = systemd_parser::internal::UnitEntry::__parse_entry(__source, #key)?.ok_or(systemd_parser::internal::Error::EntryMissingError { key: #key.to_string() })?;
            }
        }
    };

    Ok(result)
}

// pub(crate) fn gen_entry_derives(input: DeriveInput) -> syn::Result<TokenStream> {
//     todo!()
// }
