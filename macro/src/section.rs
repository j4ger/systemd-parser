use crate::{
    attribute::{impl_default, impl_default_val, impl_key},
    entry::{gen_entry_ensure, gen_entry_parse},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Meta};

pub fn gen_section_parser(input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = input;
    let key = impl_key(&attrs)?.unwrap_or((&ident).into_token_stream());
    let key_name = format!("{}", key);
    let default = impl_default(&attrs)?;

    let mut entry_ensures = Vec::new();
    let mut entry_parsers = Vec::new();
    let mut entries = Vec::new();

    if let Data::Struct(data_struct) = data {
        for entry in data_struct.fields {
            entry_ensures.push(gen_entry_ensure(&entry));
            entry_parsers.push(gen_entry_parse(&entry)?);
            entries.push(entry.ident);
        }
    } else {
        panic!("A section cannot be an enum or an union.")
    }

    let result = match default {
        true => quote! {
            impl systemd_unit_parser::UnitSection for #ident {
                fn __parse_section(source: &HashMap<String, &HashMap<String, String>>) -> Result<Self>{
                    let source = match source.get(#key) {
                        Some(inner) => inner,
                        None => { return Ok(#ident::Default()); },
                    };
                    #( #entry_parsers )*
                    Ok(Self {
                        #( #entries ),*
                    })
                }
            }
        },
        false => quote! {
            impl systemd_unit_parser::UnitSection for #ident {
                fn __parse_section(source: &HashMap<String, &HashMap<String, String>>) -> Result<Self>{
                    let source = match source.get(#key) {
                        Some(inner) => inner,
                        None => { return Err(systemd_unit_parser::error::SectionMissingError {key: #key_name}); },
                    };
                    #( #entry_parsers )*
                    Ok(Self {
                        #( #entries ),*
                    })
                }
            }
        },
    };

    Ok(result)
}
