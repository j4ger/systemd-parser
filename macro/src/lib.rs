use section::gen_section_derives;
use syn::{parse_macro_input, DeriveInput};
use unit::gen_unit_derives;

mod attribute;
mod entry;
mod section;
mod unit;

#[proc_macro_derive(UnitConfig, attributes(unit))]
pub fn derive_unit_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    gen_unit_derives(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(UnitSection, attributes(unit))]
pub fn derive_unit_section(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    gen_section_derives(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
