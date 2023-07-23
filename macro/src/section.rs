use crate::{
    attribute::SectionAttributes,
    entry::{gen_entry_ensure, gen_entry_parse},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Error, Field};

pub fn gen_section_derives(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut entry_ensures = Vec::new();
    let mut entry_parsers = Vec::new();
    let mut entries = Vec::new();

    if let Data::Struct(data_struct) = &input.data {
        for entry in &data_struct.fields {
            entry_ensures.push(gen_entry_ensure(entry));
            entry_parsers.push(gen_entry_parse(entry)?);
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
            fn __parse_section<S: AsRef<str>>(__source: &std::collections::HashMap<String, std::collections::HashMap<String, String>>, __key: S) -> systemd_parser::internal::Result<Option<Self>> {
                let __source = match __source.get(__key.as_ref()) {
                    Some(__inner) => __inner,
                    None => { return Ok(None); },
                };
                #( #entry_parsers )*
                Ok(Some(Self {
                    #( #entries ),*
                }))
            }
        }
    };

    Ok(result)
}

pub(crate) fn gen_section_parse(field: &Field) -> Result<TokenStream, Error> {
    let name = field
        .ident
        .as_ref()
        .expect("Tuple structs are not supported.");
    let ty = &field.ty;
    let attributes = SectionAttributes::parse_vec(&field.attrs)?;
    let key = attributes
        .key
        .unwrap_or((format!("{}", name)).into_token_stream());

    let result = match attributes.default {
        true => {
            let ensure = gen_section_ensure(field);
            quote! {
                #ensure
                let #name: #ty = systemd_parser::internal::UnitSection::__parse_section(__source, #key)?.unwrap_or(#ty::default());
            }
        }
        false => {
            quote! {
                let #name: #ty = systemd_parser::internal::UnitSection::__parse_section(__source, #key)?.ok_or(systemd_parser::internal::Error::EntryMissingError { key: #key.to_string() })?;
            }
        }
    };

    Ok(result)
}

fn gen_section_ensure(field: &Field) -> TokenStream {
    let ty = &field.ty;
    quote! {
        const _: fn() = || {
            fn assert_impl<T: UnitSection>() {}
            assert_impl::<#ty>();
        };
    }
}
