#[cfg(test)]
mod test_utils;
mod tool;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, parse_macro_input};

#[proc_macro_derive(Toolset, attributes(toolset))]
pub fn derive_toolset(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match tool::tool_derive_impl(&derive_input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_derive(Schema)]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match tool::schema_derive_impl(&derive_input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    match tool::tool_attribute_impl(&attr.into(), &item_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
