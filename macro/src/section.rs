use crate::{
    attribute::SectionAttributes,
    entry::{gen_entry_ensure, gen_entry_finalize, gen_entry_init, gen_entry_parse},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field, Result};

pub fn gen_section_derives(input: DeriveInput) -> Result<TokenStream> {
    let mut entry_ensures = Vec::new();
    let mut entry_inits = Vec::new();
    let mut entry_parsers = Vec::new();
    let mut entry_finalizes = Vec::new();
    let mut entries = Vec::new();

    if let Data::Struct(data_struct) = &input.data {
        for entry in &data_struct.fields {
            entry_ensures.push(gen_entry_ensure(entry));
            entry_inits.push(gen_entry_init(entry)?);
            entry_parsers.push(gen_entry_parse(entry)?);
            entry_finalizes.push(gen_entry_finalize(entry)?);
            let ident = entry.ident.as_ref().ok_or(Error::new_spanned(
                &entry,
                "An entry must have an explicit name.",
            ))?;
            entries.push(ident);
        }
    } else {
        return Err(Error::new_spanned(
            input,
            "A UnitSection cannot be an enum or an union.",
        ));
    }

    let ident = &input.ident;

    let result = quote! {
        impl systemd_parser::internal::UnitSection for #ident {
            fn __parse_section(__source: systemd_parser::internal::SectionParser) -> systemd_parser::internal::Result<Option<Self>> {
                # ( #entry_inits )*
                for __entry in __source {
                    let __pair = __entry?;
                    match __pair.0 {
                        #( #entry_parsers ),*
                        _ => {
                            log::warn!("{} is not a valid key.", __pair.0);
                        }
                    }
                }
                #( #entry_finalizes )*
                Ok(Some(Self {
                    #( #entries ),*
                }))
            }
        }
    };

    Ok(result)
}

pub(crate) fn gen_section_init(field: &Field) -> Result<TokenStream> {
    let name = field.ident.as_ref().ok_or(Error::new_spanned(
        field,
        "Tuple structs are not supported.",
    ))?;
    Ok(quote! {
        let mut #name = None;
    })
}

pub(crate) fn gen_section_parse(field: &Field) -> Result<TokenStream> {
    let name = field.ident.as_ref().ok_or(Error::new_spanned(
        field,
        "Tuple structs are not supported.",
    ))?;
    let ty = &field.ty;
    let attributes = SectionAttributes::parse_vec(&field.attrs)?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match attributes.default {
        true => {
            quote! {
                #key => {
                    const _: fn() = || {
                        fn assert_impl<T: Default>() {}
                        assert_impl::<#ty>();
                    };
                    let __value: #ty = systemd_parser::internal::UnitSection::__parse_section(__section)?
                        .unwrap_or(#ty::default());
                    #name = Some(__value);
                }
            }
        }
        false => {
            quote! {
                #key => {
                    let __value: #ty = systemd_parser::internal::UnitSection::__parse_section(__section)?
                        .ok_or(systemd_parser::internal::Error::SectionParsingError{ key: #key.to_string() })?;
                    #name = Some(__value);
                }
            }
        }
    };

    Ok(result)
}

pub(crate) fn gen_section_ensure(field: &Field) -> TokenStream {
    let ty = &field.ty;
    quote! {
        const _: fn() = || {
            fn assert_impl<T: UnitSection>() {}
            assert_impl::<#ty>();
        };
    }
}

pub(crate) fn gen_section_finalize(field: &Field) -> Result<TokenStream> {
    let name = field.ident.as_ref().ok_or(Error::new_spanned(
        field,
        "Tuple structs are not supported.",
    ))?;
    let ty = &field.ty;
    let attributes = SectionAttributes::parse_vec(&field.attrs)?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match attributes.default {
        true => {
            quote! {
                let #name: #ty = #name.unwrap_or(Default::default());
            }
        }
        false => {
            quote! {
                let #name = #name.ok_or(systemd_parser::internal::Error::SectionMissingError { key: #key.to_string()})?;
            }
        }
    };

    Ok(result)
}
