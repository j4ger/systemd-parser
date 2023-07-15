use syn::{parse_macro_input, DeriveInput};

mod attribute;
mod entry;
mod section;
mod unit;

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
