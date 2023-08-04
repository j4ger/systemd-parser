use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field, Result};

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

pub(crate) fn gen_entry_init(field: &Field) -> Result<TokenStream> {
    let name = field.ident.as_ref().ok_or(Error::new_spanned(
        field,
        "Tuple structs are not supported.",
    ))?;
    Ok(quote! {
        let mut #name = None;
    })
}

pub(crate) fn gen_entry_parse(field: &Field) -> Result<TokenStream> {
    let name = field.ident.as_ref().ok_or(Error::new_spanned(
        field,
        "Tuple structs are not supported.",
    ))?;
    let ty = &field.ty;
    let attributes = EntryAttributes::parse_vec(&field.attrs)?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match attributes.default {
        Some(default) => {
            let default = transform_default(ty, &default)?;
            quote! {
                #key => {
                    let __value: #ty = systemd_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str())
                        .unwrap_or(#default);
                    #name = Some(__value);
                }
            }
        }
        None => {
            quote! {
                #key => {
                    let __value: #ty = systemd_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str())
                        .map_err(|_| systemd_parser::internal::Error::ValueParsingError { key: #key.to_string(), value: __pair.1.to_string() })?;
                    #name = Some(__value);
                }
            }
        }
    };

    Ok(result)
}

pub(crate) fn gen_entry_finalize(field: &Field) -> Result<TokenStream> {
    let name = field.ident.as_ref().ok_or(Error::new_spanned(
        field,
        "Tuple structs are not supported.",
    ))?;
    let ty = &field.ty;
    let attributes = EntryAttributes::parse_vec(&field.attrs)?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match attributes.default {
        Some(default) => {
            let default = transform_default(ty, &default)?;
            quote! {
                let #name = #name.unwrap_or(#default);
            }
        }
        None => {
            quote! {
                let #name = #name.ok_or(systemd_parser::internal::Error::EntryMissingError { key: #key.to_string()})?;
            }
        }
    };
    Ok(result)
}

pub(crate) fn gen_entry_derives(input: DeriveInput) -> Result<TokenStream> {
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
