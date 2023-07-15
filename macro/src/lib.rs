mod attribute;
mod entry;
mod section;
mod unit;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, spanned::Spanned, Attribute, DeriveInput, Error, Expr, Ident,
    Meta, Token, TypePath, Visibility,
};

#[proc_macro_derive(UnitConfig, attributes(unit))]
pub fn derive_unit_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    todo!()
}

#[proc_macro_derive(UnitSection, attributes(unit))]
pub fn derive_unit_section(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    todo!()
}
