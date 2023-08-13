use entry::gen_entry_derives;
use section::gen_section_derives;
use syn::{parse_macro_input, DeriveInput};
use unit::gen_unit_derives;

mod attribute;
mod entry;
mod section;
mod transform_default;
mod type_transform;
mod unit;

#[proc_macro_derive(UnitConfig, attributes(section))]
pub fn derive_unit_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    gen_unit_derives(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(UnitSection, attributes(entry))]
pub fn derive_unit_section(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    gen_section_derives(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(UnitEntry)]
pub fn derive_unit_entry(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    gen_entry_derives(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
