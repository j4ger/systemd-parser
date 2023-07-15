use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, spanned::Spanned, Attribute, DeriveInput, Error, Expr, Ident,
    Meta, Token, TypePath, Visibility,
};

/// an unit definition
struct UnitDefinition {}

// ensure each section derives `UnitSection`
