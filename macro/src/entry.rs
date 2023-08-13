use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field, Result};

use crate::{
    attribute::EntryAttributes,
    transform_default::transform_default,
    type_transform::{is_option, is_vec},
};

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
    let attributes = EntryAttributes::parse_vec(&field.attrs)?;
    Ok(match attributes.multiple {
        false => quote! {
            let mut #name = None;
        },
        true => quote! {
            let mut #name = Vec::new();
        },
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

    let result = match (attributes.default, attributes.multiple) {
        (_, true) => {
            quote! {
                #key => {
                    if __pair.1.as_str().is_empty() {
                        #name.clear();
                        continue;
                    }
                    match systemd_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str()){
                        Ok(__inner) => {
                            #name.push(__inner);
                        }
                        Err(_) => {
                            log::warn!("Failed to parse {} for key {}, ignoring.", __pair.0, __pair.1);
                        }
                    }
                }
            }
        }
        (Some(default), false) => {
            let default = transform_default(ty, &default)?;
            quote! {
                #key => {
                    let __value = systemd_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str())
                        .unwrap_or(#default);
                    #name = Some(__value);
                }
            }
        }
        (None, false) => {
            quote! {
                #key => {
                    let __value = systemd_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str())
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

    let result = match (attributes.default, attributes.multiple, attributes.optional) {
        (_, true, false) => {
            if !is_vec(ty) {
                return Err(Error::new_spanned(
                    ty,
                    "`multiple` attributed fields should be `Vec`s.",
                ));
            }
            quote! {
                if #name.is_empty() {
                    log::warn!("{} is defined but no value is present.", #key);
                }
            }
        }
        (Some(default), false, false) => {
            let default = transform_default(ty, &default)?;
            quote! {
                let #name = #name.unwrap_or(#default);
            }
        }
        (None, false, false) => {
            quote! {
                let #name = #name.ok_or(systemd_parser::internal::Error::EntryMissingError { key: #key.to_string()})?;
            }
        }
        (_, _, true) => {
            if !is_option(ty) {
                return Err(Error::new_spanned(
                    ty,
                    "`optional` attributed fields should be `Option`s.",
                ));
            }
            quote! {}
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
