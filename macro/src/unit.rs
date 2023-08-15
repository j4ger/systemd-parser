use crate::section::{
    gen_section_ensure, gen_section_finalize, gen_section_init, gen_section_parse,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error};

// ensure each section derives `UnitSection`
pub fn gen_unit_derives(input: DeriveInput) -> syn::Result<TokenStream> {
    let mut sections = Vec::new();
    let mut section_ensures = Vec::new();
    let mut section_inits = Vec::new();
    let mut section_parsers = Vec::new();
    let mut section_finalizes = Vec::new();

    if let Data::Struct(data_struct) = &input.data {
        for entry in &data_struct.fields {
            section_ensures.push(gen_section_ensure(&entry));
            section_inits.push(gen_section_init(&entry)?);
            section_parsers.push(gen_section_parse(&entry)?);
            section_finalizes.push(gen_section_finalize(&entry)?);
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
         impl unit_parser::internal::UnitConfig for #ident {
            fn __parse_unit(__source: unit_parser::internal::UnitParser) -> unit_parser::internal::Result<Self> {
                #( #section_ensures )*
                #( #section_inits )*
                for __section in __source {
                    let __section = __section?;
                    match __section.name {
                        #( #section_parsers ),*
                        _ => {
                            log::warn!("{} is not a valid section.", __section.name);
                        }
                    }
                }
                #( #section_finalizes )*
                Ok(Self {
                    #( #sections ),*
                })
            }
        }
    };

    Ok(result)
}
