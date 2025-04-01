use convert_case::{Case, Casing};
use darling::{FromMeta, ast::NestedMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, PatType};

use super::ToolMeta;

pub fn tool_attribute_impl(
    attr: &TokenStream,
    item: &ItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
    let attr_args = NestedMeta::parse_meta_list(attr.clone())?;

    let params = match item.sig.inputs.first() {
        Some(syn::FnArg::Typed(PatType { ty, .. })) => match ty.as_ref() {
            syn::Type::Reference(r) => match r.elem.as_ref() {
                syn::Type::Path(p) if !p.path.segments.is_empty() => Some(p.path.clone()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    };

    let ToolMeta { name, description } = ToolMeta::from_list(&attr_args)?;
    let ItemFn { sig, block, .. } = item;

    let struct_ident = syn::Ident::new(
        &format!("{}Toolset", name.to_case(Case::Pascal)),
        proc_macro2::Span::call_site(),
    );
    let toolset_trait_ident = syn::Ident::new(
        &format!("{}Invoke", name.to_case(Case::Pascal)),
        proc_macro2::Span::call_site(),
    );

    let tool_name = name.to_case(Case::Title);

    let args_ident = params.clone().into_iter();
    let params_ident = params.clone().into_iter();

    let statements = block.stmts.clone();
    let function_identifier = sig.ident.clone();

    Ok(quote! {
        #[derive(meerai_macros::Toolset)]
        #[toolset(
            name = #name,
            tool(name = #tool_name, description = #description #(, params = #args_ident)*)
        )]
        pub struct #struct_ident;

        #[meerai_core::async_trait]
        impl #toolset_trait_ident for #struct_ident {
            async fn #function_identifier(&self, #(args: &#params_ident)*) -> Result<meerai_core::ToolOutput, meerai_core::ToolError> {
                let __result = {
                    #(#statements)*
                };
                return __result;

            }
        }
    })
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use super::*;

    #[test]
    #[allow(dead_code)]
    fn test_snapshot_attribute_with_params() {
        pub struct HelloAttributeArgs {
            text: String,
        }
        let args = quote! {
            name = "hello",
            description = "hello",
        };
        let input: ItemFn = parse_quote! {
            pub async fn hello_attribute(args: &HelloAttributeArgs) -> Result<ToolOutput, ToolError> {
                return Ok(ToolOutput::Text("hello".into()))
            }
        };

        let output = tool_attribute_impl(&args, &input).unwrap();

        insta::assert_snapshot!(crate::test_utils::pretty_macro_output(&output));
    }

    #[test]
    #[allow(dead_code)]
    fn test_snapshot_attribute_without_params() {
        let args = quote! {
            name = "hello",
            description = "hello",
        };
        let input: ItemFn = parse_quote! {
            pub async fn hello_attribute() -> Result<ToolOutput, ToolError> {
                return Ok(ToolOutput::Text("hello".into()))
            }
        };

        let output = tool_attribute_impl(&args, &input).unwrap();

        insta::assert_snapshot!(crate::test_utils::pretty_macro_output(&output));
    }
}
