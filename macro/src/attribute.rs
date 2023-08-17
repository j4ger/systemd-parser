use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Error, Expr, LitStr, Token};

pub(crate) struct SectionAttributes {
    pub(crate) default: bool,
    pub(crate) key: Option<TokenStream>,
    pub(crate) must: bool,
}

impl Default for SectionAttributes {
    fn default() -> Self {
        Self {
            default: false,
            key: None,
            must: false,
        }
    }
}

impl SectionAttributes {
    pub(crate) fn parse_vec(input: &Vec<Attribute>) -> syn::Result<Self> {
        let mut result = SectionAttributes::default();
        for attribute in input {
            if attribute.path().is_ident("section") {
                attribute.parse_nested_meta(|nested| {
                    if nested.path.is_ident("default") {
                        result.default = true;
                        Ok(())
                    } else if nested.path.is_ident("key") {
                        nested.input.parse::<Token![=]>()?;
                        let value: LitStr = nested.input.parse()?;
                        result.key = Some(value.into_token_stream());
                        Ok(())
                    } else if nested.path.is_ident("must") {
                        result.must = true;
                        Ok(())
                    } else {
                        Err(Error::new_spanned(attribute, "Not a valid attribute."))
                    }
                })?;
            }
        }
        Ok(result)
    }
}

pub(crate) struct EntryAttributes {
    pub(crate) default: Option<Expr>,
    pub(crate) key: Option<TokenStream>,
    pub(crate) multiple: bool,
    pub(crate) must: bool,
}

impl Default for EntryAttributes {
    fn default() -> Self {
        Self {
            default: None,
            key: None,
            multiple: false,
            must: false,
        }
    }
}

impl EntryAttributes {
    pub(crate) fn parse_vec(input: &Vec<Attribute>) -> syn::Result<Self> {
        let mut result = EntryAttributes::default();
        for attribute in input {
            if attribute.path().is_ident("entry") {
                attribute.parse_nested_meta(|nested| {
                    if nested.path.is_ident("default") {
                        nested.input.parse::<Token![=]>()?;
                        let value: Expr = nested.input.parse()?;
                        result.default = Some(value);
                        Ok(())
                    } else if nested.path.is_ident("key") {
                        nested.input.parse::<Token![=]>()?;
                        let value: LitStr = nested.input.parse()?;
                        result.key = Some(value.into_token_stream());
                        Ok(())
                    } else if nested.path.is_ident("multiple") {
                        result.multiple = true;
                        Ok(())
                    } else if nested.path.is_ident("must") {
                        result.must = true;
                        Ok(())
                    } else {
                        Err(Error::new_spanned(attribute, "Not a valid attribute."))
                    }
                })?;
            }
        }
        Ok(result)
    }
}

pub(crate) struct UnitAttributes {
    pub(crate) suffix: Option<LitStr>,
}

impl Default for UnitAttributes {
    fn default() -> Self {
        Self { suffix: None }
    }
}

impl UnitAttributes {
    pub(crate) fn parse_vec(input: &Vec<Attribute>) -> syn::Result<Self> {
        let mut result = UnitAttributes::default();
        for attribute in input {
            if attribute.path().is_ident("unit") {
                attribute.parse_nested_meta(|nested| {
                    if nested.path.is_ident("suffix") {
                        nested.input.parse::<Token![=]>()?;
                        let value: LitStr = nested.input.parse()?;
                        result.suffix = Some(value);
                        Ok(())
                    } else {
                        Err(Error::new_spanned(attribute, "Not a valid attribute."))
                    }
                })?;
            }
        }
        Ok(result)
    }
}
