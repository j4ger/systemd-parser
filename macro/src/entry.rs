use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    token::Brace,
    Attribute, DeriveInput, Error, Expr, Ident, Meta, Token, TypePath, Visibility,
};

/// a line of entry definition placed in a section definition
pub(crate) struct EntryDefinition {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    ty: TypePath,
}

impl Parse for EntryDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: TypePath = input.parse()?;

        // Optional for the last one?
        input.parse::<Token![,]>()?;
        Ok(EntryDefinition {
            attributes,
            visibility,
            name,
            ty,
        })
    }
}

// for entries
impl EntryDefinition {
    fn parse_vec(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut result = Vec::new();
        while !input.peek(Brace) {
            result.push(input.parse()?);
        }
        Ok(result)
    }

    fn gen_ensure(&self) -> TokenStream {
        let ty = &self.ty;
        quote! {
            const _: fn() = || {
                fn assert_impl<T: UnitEntry>() {}
                assert_impl::<ty>();
            };
        }
    }

    // [important!] run gen_parse first to filter out used attributes
    // before calling gen definition
    fn gen_parse(&mut self) -> Result<TokenStream, Error> {
        let default = match self
            .attributes
            .iter()
            .position(|x| x.path().is_ident("default"))
        {
            None => None,
            Some(pos) => {
                let target = self.attributes.remove(pos);
                if let Meta::NameValue(pair) = target.meta {
                    Some(pair.value)
                } else {
                    let span = target.span();
                    return Err(Error::new(span, "not a valid default attribute"));
                }
            }
        };

        let key = match self
            .attributes
            .iter()
            .position(|x| x.path().is_ident("key"))
        {
            None => self.name.to_token_stream(),
            Some(pos) => {
                let target = self.attributes.remove(pos);
                let span = target.span();
                if let Meta::NameValue(pair) = target.meta {
                    let value = pair.value;
                    if let Expr::Lit(_) = value {
                        value.to_token_stream()
                    } else {
                        return Err(Error::new(span, "not a valid key name"));
                    }
                } else {
                    return Err(Error::new(span, "not a valid key name attribute"));
                }
            }
        };

        let name = &self.name;

        let result = match default {
            None => {
                // error if not found
                // should be done in lib, using different parsing functions
                quote! {
                    let #name = Self::__parse( __source, "#key")?;
                }
            }
            Some(default) => {
                // fallback to default if not found
                quote! {
                    let #name = Self::__parse_with_default( __source, "#key", #default)?;
                }
            }
        };

        Ok(result)
    }

    fn gen_definition(&self) -> TokenStream {
        // is this really necessary?
        // this would retain attributes that are not used by the crate
        // but are parsed for the name
        let attributes = self
            .attributes
            .iter()
            .map(|x| x.into_token_stream())
            .collect::<TokenStream>();
        let visibility = &self.visibility;
        let name = &self.name;
        let ty = &self.ty;
        quote! {
            #attributes
            #visibility #name: #ty,
        }
    }
}
