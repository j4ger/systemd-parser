use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field};

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

pub(crate) fn gen_entry_derives(input: DeriveInput) -> syn::Result<TokenStream> {
    if let Data::Enum(inner) = input.data {
        let ident = &input.ident;
        let mut match_arms = Vec::new();

        for variant in inner.variants.iter() {
            let name = &variant.ident;
            let value = format!("{}", name);
            // use discrimnant for alt-key
            let result = quote! {
                #value => Ok(Self::#name)
            };
            match_arms.push(result);
        }

        Ok(quote! {
            impl systemd_parser::internal::UnitEntry for #ident {
                type Error = ();
                fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error> {
                    match input.as_ref() {
                        #( #match_arms ,)*
                        _ => Err(()),
                    }
                }
            }

            impl systemd_parser::internal::EntryInner for #ident {}
        })
    } else {
        Err(Error::new_spanned(
            input,
            "UnitEntry can only be derived on enum definitions.",
        ))
    }
}
