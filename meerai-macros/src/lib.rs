#[cfg(test)]
mod test_utils;
mod tool;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, parse_macro_input};

#[proc_macro_derive(Toolset, attributes(toolset))]
pub fn derive_toolset(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = parse_macro_input!(input);
    match tool::tool_derive_impl(&derive_input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn tool(agrs: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn: ItemFn = parse_macro_input!(item);
    match tool::tool_attribute_impl(&agrs.into(), &item_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
