use crate::{
    attribute::{impl_default, impl_key},
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
        impl systemd_unit_parser::UnitSection for #ident {
            fn __parse_section<S: AsRef<str>>(source: &std::collections::HashMap<String, std::collections::HashMap<String, String>>, key: S) -> Option<Self> {
                let __source = match source.get(key) {
                    Some(inner) => inner,
                    None => { return Err(systemd_unit_parser::error::SectionMissingError {key}); },
                };
                #( #entry_parsers )*
                Ok(Self {
                    #( #entries ),*
                })
            }
        }
    };

    Ok(result)
}

pub(crate) fn gen_section_parse(field: &Field) -> Result<TokenStream, Error> {
    let name = &field
        .ident
        .as_ref()
        .expect("Tuple structs are not supported.");
    let ty = &field.ty;
    let key = impl_key(&field.attrs)?.unwrap_or((&field.ident).into_token_stream());
    let default = impl_default(&field.attrs)?;

    let key_name = format!("{}", key);

    let result = match default {
        true => {
            let ensure = gen_section_ensure(field);
            quote! {
                #ensure
                let #name = #ty::__parse_section(__source, #key_name)?.unwrap_or(#ty::default());
            }
        }
        false => {
            quote! {
                let #name = #ty::__parse_section(__source, #key_name)?.ok_or(systemd_unit_parser::error::EntryMissingError { key: #key_name })?;
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
