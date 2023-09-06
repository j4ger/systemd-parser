use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field, Result};

use crate::{
    attribute::EntryAttributes,
    transform_default::transform_default,
    type_transform::{extract_type_from_option, extract_type_from_vec},
};

pub(crate) fn gen_entry_ensure(field: &Field) -> Result<TokenStream> {
    let mut ty = &field.ty;
    let attribute = EntryAttributes::parse_vec(field, None)?;
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
    let attributes = EntryAttributes::parse_vec(field, None)?;
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
    let attributes = EntryAttributes::parse_vec(field, Some(ty))?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match (
        attributes.default,
        attributes.multiple,
        attributes.subdir,
        attributes.must,
    ) {
        // unreachable
        (Some(_), _, _, true) | (_, true, _, true) | (_, false, Some(_), _) => unreachable!(),
        // add to Vec
        (_, true, None, _) => {
            quote! {
                #key => {
                    if __pair.1.as_str().is_empty() {
                        #name.clear();
                        continue;
                    }
                    for __part in __pair.1.split_ascii_whitespace(){
                        match unit_parser::internal::UnitEntry::parse_from_str(__part){
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
        }
        // add to Vec, as well as subdirs
        (_, true, Some(subdir), _) => {
            quote! {
                #key => {
                    if __pair.1.as_str().is_empty() {
                        #name.clear();
                        continue;
                    }
                    for __part in __pair.1.split_ascii_whitespace(){
                        match unit_parser::internal::UnitEntry::parse_from_str(__part){
                            Ok(__inner) => {
                                #name.push(__inner);
                            }
                            Err(_) => {
                                log::warn!("Failed to parse {} for key {}, ignoring.", __pair.0, __pair.1);
                            }
                        }
                    }
                    let __subdirs = __subdir_parser.__parse_subdir(#subdir);
                    #name.extend_from_slice(&__subdirs);
                }
            }
        }
        // set as Some if Ok
        (_, false, None, false) => {
            quote! {
                #key => {
                    if let Ok(__value) = unit_parser::internal::UnitEntry::parse_from_str(__pair.1.as_str()) {
                        #name = Some(__value);
                    }
                }
            }
        }
        // throw Error
        (None, false, None, true) => {
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
    let attributes = EntryAttributes::parse_vec(field, None)?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match (attributes.default, attributes.multiple, attributes.must) {
        // invalid
        (Some(_), _, true) | (_, true, true) => unreachable!(),
        // apply default if empty
        (Some(default), true, false) => {
            quote! {
                if #name.is_empty() {
                    #name = #default;
                }
            }
        }
        // leave unchanged (`Vec` and `Option`)
        (None, true, false) | (None, false, false) => {
            quote! {}
        }
        // unwrap to default
        (Some(default), false, false) => {
            let default = transform_default(ty, &default)?;
            quote! {
                let #name = #name.unwrap_or(#default);
            }
        }
        // throw Error
        (None, false, true) => {
            quote! {
                let #name = #name.ok_or(unit_parser::internal::Error::EntryMissingError { key: #key.to_string()})?;
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
            // TODO: support for alt-key
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
    let attributes = EntryAttributes::parse_vec(field, None)?;

    let result = match (attributes.must, attributes.multiple, attributes.default) {
        // invalid
        (true, _, Some(_)) | (true, true, _) => unreachable!(),
        // append
        // TODO: or should it overwrite?
        (false, true, _) => {
            quote! {
                __from.#name.extend_from_slice(&#name);
            }
        }
        // set (as is) if not None
        (false, false, None) => {
            quote! {
                if #name.is_some() {
                    __from.#name = #name;
                }
            }
        }
        // set if not None
        (_, false, _) => {
            quote! {
                if let Some(__inner) = #name {
                    __from.#name = __inner;
                }
            }
        }
    };

    Ok(result)
}
