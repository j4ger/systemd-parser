use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, Error, Expr, LitStr, Token};

pub(crate) struct SectionAttributes {
    pub(crate) default: bool,
    pub(crate) key: Option<TokenStream>,
    pub(crate) optional: bool,
}

impl Default for SectionAttributes {
    fn default() -> Self {
        Self {
            default: false,
            key: None,
            optional: false,
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
                        if result.optional {
                            return Err(Error::new_spanned(
                                attribute,
                                "`optional` and `default` attribute cannot co-exist.",
                            ));
                        }
                        result.default = true;
                        Ok(())
                    } else if nested.path.is_ident("key") {
                        nested.input.parse::<Token![=]>()?;
                        let value: LitStr = nested.input.parse()?;
                        result.key = Some(value.into_token_stream());
                        Ok(())
                    } else if nested.path.is_ident("optional") {
                        if result.default {
                            return Err(Error::new_spanned(
                                attribute,
                                "`optional` and `default` attribute cannot co-exist.",
                            ));
                        }
                        result.optional = true;
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
    pub(crate) optional: bool,
}

impl Default for EntryAttributes {
    fn default() -> Self {
        Self {
            default: None,
            key: None,
            multiple: false,
            optional: false,
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
                        if result.optional {
                            return Err(Error::new_spanned(
                                attribute,
                                "`optional` and `default` attribute cannot co-exist.",
                            ));
                        }
                        result.default = Some(value);
                        Ok(())
                    } else if nested.path.is_ident("key") {
                        nested.input.parse::<Token![=]>()?;
                        let value: LitStr = nested.input.parse()?;
                        result.key = Some(value.into_token_stream());
                        Ok(())
                    } else if nested.path.is_ident("multiple") {
                        if result.optional {
                            return Err(Error::new_spanned(
                                attribute,
                                "`optional` and `multiple` attribute cannot co-exist.",
                            ));
                        }
                        result.multiple = true;
                        Ok(())
                    } else if nested.path.is_ident("optional") {
                        if result.default.is_some() {
                            return Err(Error::new_spanned(
                                attribute,
                                "`optional` and `default` attribute cannot co-exist.",
                            ));
                        }
                        if result.multiple {
                            return Err(Error::new_spanned(
                                attribute,
                                "`optional` and `multiple` attribute cannot co-exist.",
                            ));
                        }
                        result.optional = true;
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
