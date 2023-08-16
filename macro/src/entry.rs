use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field, Result};

use crate::{
    attribute::EntryAttributes,
    transform_default::transform_default,
    type_transform::{extract_type_from_option, extract_type_from_vec, is_option, is_vec},
};

pub(crate) fn gen_entry_ensure(field: &Field) -> Result<TokenStream> {
    let mut ty = &field.ty;
    let attribute = EntryAttributes::parse_vec(&field.attrs)?;
    if attribute.multiple {
        ty = extract_type_from_vec(ty)?;
    } else if (!attribute.must) & (attribute.default.is_none()) {
        ty = extract_type_from_option(ty)?;
    }
    Ok(quote! {
        const _: fn() = || {
            fn assert_impl<T: UnitEntry>() {}
            assert_impl::<#ty>();
        };
    })
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
                    match unit_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str()){
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
                    let __value = unit_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str())
                        .unwrap_or(#default);
                    #name = Some(__value);
                }
            }
        }
        (None, false) => {
            quote! {
                #key => {
                    let __value = unit_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str())
                        .map_err(|_| unit_parser::internal::Error::ValueParsingError { key: #key.to_string(), value: __pair.1.to_string() })?;
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

    let result = match (attributes.default, attributes.multiple, attributes.must) {
        (None, true, true) => {
            if !is_vec(ty) {
                return Err(Error::new_spanned(
                    ty,
                    "`multiple` attributed fields should be `Vec`s.",
                ));
            }
            quote! {
                if #name.is_empty() {
                    return Err(unit_parser::internal::Error::EntryMissingError { key: #key.to_string() });
                }
            }
        }
        (Some(default), true, _) => {
            if !is_vec(ty) {
                return Err(Error::new_spanned(
                    ty,
                    "`multiple` attributed fields should be `Vec`s.",
                ));
            }
            quote! {
                if #name.is_empty() {
                    #name = #default;
                }
            }
        }
        (None, true, false) => {
            if !is_vec(ty) {
                return Err(Error::new_spanned(
                    ty,
                    "`multiple` attributed fields should be `Vec`s.",
                ));
            }
            quote! {
                if #name.is_empty() {
                    log::warn!("{} is defined but no value is found.", #key);
                }
            }
        }
        (Some(default), false, _) => {
            let default = transform_default(ty, &default)?;
            quote! {
                let #name = #name.unwrap_or(#default);
            }
        }
        (None, false, true) => {
            quote! {
                let #name = #name.ok_or(unit_parser::internal::Error::EntryMissingError { key: #key.to_string()})?;
            }
        }
        (None, _, false) => {
            if !is_option(ty) {
                return Err(Error::new_spanned(
                    ty,
                    "Fields without either `default` or `must` attributes should be `Option`s.",
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
            impl unit_parser::internal::UnitEntry for #ident {
                type Error = ();
                fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error> {
                    match input.as_ref() {
                        #( #match_arms ,)*
                        _ => Err(()),
                    }
                }
            }

            impl unit_parser::internal::EntryInner for #ident {}
        })
    } else {
        Err(Error::new_spanned(
            input,
            "UnitEntry can only be derived on enum definitions.",
        ))
    }
}

pub(crate) fn gen_entry_patch(field: &Field) -> Result<TokenStream> {
    let name = field.ident.as_ref().ok_or(Error::new_spanned(
        field,
        "Tuple structs are not supported.",
    ))?;
    let attributes = EntryAttributes::parse_vec(&field.attrs)?;

    let result = match (attributes.must, attributes.multiple) {
        (true, true) => {
            quote! {
                if !#name.is_empty() {
                    __from.#name = #name;
                }
            }
        }
        (false, true) => {
            quote! {
                __from.#name = #name;
            }
        }
        (true, false) => {
            quote! {
                if let Some(__inner) = #name {
                    __from.#name = __inner;
                }
            }
        }
        (false, false) => {
            quote! {
                if #name.is_some() {
                    __from.#name = #name;
                }
            }
        }
    };

    Ok(result)
}
