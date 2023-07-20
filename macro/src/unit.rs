use crate::section::gen_section_parse;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error};

// ensure each section derives `UnitSection`
pub fn gen_unit_derives(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut sections = Vec::new();
    let mut section_parsers = Vec::new();

    if let Data::Struct(data_struct) = &input.data {
        for entry in &data_struct.fields {
            section_parsers.push(gen_section_parse(&entry)?);
            let ident = entry.ident.as_ref().ok_or(Error::new_spanned(
                &entry,
                "An entry must have an explicit name.",
            ))?;
            sections.push(ident);
        }
    } else {
        return Err(Error::new_spanned(
            &input,
            "A UnitConfig cannot be an enum or an union.",
        ));
    }

    let ident = &input.ident;

    let result = quote! {
         impl systemd_parser::internal::UnitConfig for #ident {
            fn parse(__source: &std::collections::HashMap<String, std::collections::HashMap<String, String>>) -> systemd_parser::internal::Result<Self> {
                #( #section_parsers )*
                Ok(Self {
                    #( #sections ),*
                })
            }
        }
    };

    Ok(result)
}
