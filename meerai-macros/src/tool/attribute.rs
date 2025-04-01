use darling::{Error, FromMeta, ast::NestedMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Pat, PatType};

use super::common::ToolArgs;

pub fn tool_attribute_impl(
    args: &TokenStream,
    item: &ItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
    let tool_args = create_tool_args(args, item)?;
    Ok(quote! {})
}

fn create_tool_args(args: &TokenStream, item: &ItemFn) -> Result<ToolArgs, Error> {
    let attr_args = NestedMeta::parse_meta_list(args.clone())?;
    let mut tool_args = ToolArgs::from_list(&attr_args)?;

    for args in item.sig.inputs.iter() {
        if let syn::FnArg::Typed(PatType { pat, ty, .. }) = args {
            if let Pat::Ident(ident) = &**pat {
                //             if let Some(ident) = type_path.path.get_ident() {
                //                 tool_args.params.push(Param {
                //                     name: ident.to_string(),
                //                     rust_type: type_path.clone(),
                //                     json_type: "string".to_string(), // Default to string
                //                 });
                //             }
            }
        }
    }

    Ok(tool_args)
}
