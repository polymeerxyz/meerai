use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use quote::quote;
use syn::DeriveInput;

use super::ToolsetDerive;

pub fn tool_derive_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let mut parsed = ToolsetDerive::from_derive_input(input)?;
    if parsed.toolset.name.is_empty() {
        parsed.toolset.name = parsed.ident.to_string().to_case(Case::Pascal);
    }

    let struct_ident = &parsed.ident;
    let toolset_name = &parsed.toolset.name.to_case(Case::Snake);

    let toolset_trait_ident = syn::Ident::new(
        &format!("{}Invoke", toolset_name.to_case(Case::Pascal)),
        proc_macro2::Span::call_site(),
    );
    let toolset_trait = build_toolset_trait(&parsed, &toolset_trait_ident);
    let definition_fn = build_definition_fn(&parsed);
    let contain_fn = build_contain_fn(&parsed);
    let invoke_fn = build_invoke_fn(&parsed);

    Ok(quote! {
        #toolset_trait

        #[meerai_core::async_trait]
        impl meerai_core::Toolset for #struct_ident where Self: #toolset_trait_ident {
            fn name(&self) -> String {
                #toolset_name.to_string()
            }

            #definition_fn

            #contain_fn

            #invoke_fn
        }
    })
}

fn build_toolset_trait(
    derived: &ToolsetDerive,
    trait_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    let fns = derived
        .toolset
        .tools
        .iter()
        .map(|tool| {
            let fn_ident = syn::Ident::new(&tool.get_fn_name(), proc_macro2::Span::call_site());
            if tool.params.is_none() {
                quote! {
                    async fn #fn_ident(&self) -> Result<meerai_core::ToolOutput, meerai_core::ToolError>;
                }
            } else {
                let args_struct_ident = tool.args_struct_ident();
                quote! {
                    async fn #fn_ident(&self, args: &#args_struct_ident) -> Result<meerai_core::ToolOutput, meerai_core::ToolError>;
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        #[meerai_core::async_trait]
        trait #trait_ident {
            #(#fns)*
        }
    }
}

fn build_definition_fn(derived: &ToolsetDerive) -> proc_macro2::TokenStream {
    let definitions = derived
        .toolset
        .tools
        .iter()
        .map(|tool| {
            let fn_name = build_fn_name(&derived.toolset.name, &tool.get_fn_name());
            let description = &tool.description;
            let parameters = if let Some(params) = &tool.params {
                let params_ident = params.get_ident().unwrap();
                quote! {
                    #params_ident::json_schema()
                }
            } else {
                quote! {
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                        }
                    })
                }
            };

            quote! {
                meerai_core::ToolDefinition {
                    r#type: "function".to_string(),
                    name: #fn_name.to_string(),
                    description: #description.to_string(),
                    parameters: #parameters,
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        fn definition(&self) -> Vec<meerai_core::ToolDefinition> {
            vec![#(#definitions),*]
        }
    }
}

fn build_contain_fn(derived: &ToolsetDerive) -> proc_macro2::TokenStream {
    let fn_names = derived
        .toolset
        .tools
        .iter()
        .map(|tool| build_fn_name(&derived.toolset.name, &tool.get_fn_name()))
        .collect::<Vec<String>>();

    quote! {
        fn contain(&self, fn_name: &str) -> bool {
            match fn_name {
                #(#fn_names => true,)*
                _ => false,
            }
        }
    }
}

fn build_invoke_fn(derived: &ToolsetDerive) -> proc_macro2::TokenStream {
    let invoke_matches = derived.toolset.tools.iter().map(|tool| {
        let fn_name = build_fn_name(&derived.toolset.name, &tool.get_fn_name());
        let fn_ident = syn::Ident::new(&tool.get_fn_name(), proc_macro2::Span::call_site());
        let args_struct_ident = tool.args_struct_ident();

        if tool.params.is_none() {
            quote! {
                #fn_name => {
                    let result = self.#fn_ident().await?;
                    Ok(result)
                }
            }
        } else {
            quote! {
                #fn_name => {
                    let args: #args_struct_ident = serde_json::from_str(args).map_err(|e| meerai_core::ToolError::WrongArguments(e))?;
                    let result = self.#fn_ident(&args).await?;
                    Ok(result)
                }
            }
        }
  }).collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        async fn invoke(&self, fn_name: &str, args: &str) -> Result<meerai_core::ToolOutput, meerai_core::ToolError> {
            match fn_name {
                #(#invoke_matches)*
                _ => Err(meerai_core::ToolError::InvalidFunctionName(fn_name.to_string())),
            }
        }
    }
}

fn build_fn_name(tool_name: &str, fn_name: &str) -> String {
    format!("{}-{}", tool_name.to_case(Case::Snake), fn_name)
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_snapshot_derive_with_params() {
        let input: DeriveInput = parse_quote! {
            #[toolset(
                name = "Hello Derive",
                tool(
                    name = "hello",
                    description = "Hello",
                    params = HelloParams,
                ))
            ]
            pub struct HelloDerive {
                my_dependency: String
            }
        };

        let output = tool_derive_impl(&input).unwrap();

        insta::assert_snapshot!(crate::test_utils::pretty_macro_output(&output));
    }

    #[test]
    fn test_snapshot_derive_without_params() {
        let input: DeriveInput = parse_quote! {
            #[toolset(
                name = "Hello Derive",
                tool(
                    name = "hello",
                    description = "Hello",
                ))
            ]
            pub struct HelloDerive {
                my_dependency: String
            }
        };

        let output = tool_derive_impl(&input).unwrap();

        insta::assert_snapshot!(crate::test_utils::pretty_macro_output(&output));
    }
}
