use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use quote::quote;
use syn::DeriveInput;

use super::ToolsetDerive;

pub fn tool_derive_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let parsed = ToolsetDerive::from_derive_input(input)?;
    let struct_ident = &parsed.ident;
    let toolset_name = &parsed.toolset.name.to_case(Case::Snake);

    let args_structs = extract_args_struct(&parsed);
    let toolset_trait = build_toolset_trait(&parsed);
    let definition_fn = build_definition_fn(&parsed);
    let contain_fn = build_contain_fn(&parsed);
    let invoke_fn = build_invoke_fn(&parsed);

    Ok(quote! {
        use meerai_core::{ToolDefinition, ToolError, ToolOutput, Toolset};

        #(#args_structs)*

        #toolset_trait

        #[async_trait::async_trait]
        impl Toolset for #struct_ident where Self: Invoke {
            fn name(&self) -> String {
                #toolset_name.to_string()
            }

            #definition_fn

            #contain_fn

            #invoke_fn
        }
    })
}

fn extract_args_struct(derived: &ToolsetDerive) -> Vec<proc_macro2::TokenStream> {
    derived
        .toolset
        .tools
        .iter()
        .map(|tool| {
            let args_struct = tool.args_struct();

            quote! {
                #args_struct
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>()
}

fn build_toolset_trait(derived: &ToolsetDerive) -> proc_macro2::TokenStream {
    let trait_ident = syn::Ident::new(
        "Invoke",
        proc_macro2::Span::call_site(),
    );

    let fns = derived
      .toolset
      .tools
      .iter()
      .map(|tool| {
          let fn_ident = syn::Ident::new(&tool.fn_name, proc_macro2::Span::call_site());
          if tool.params.is_empty() {
              return quote! {
                  async fn #fn_ident(&self) -> Result<ToolOutput, ToolError>;
              };
          }
          
          let args_struct_ident = tool.args_struct_ident();

          quote! {
              async fn #fn_ident(&self, args: &#args_struct_ident) -> Result<ToolOutput, ToolError>;
          }
      })
      .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        #[async_trait::async_trait]
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
            let fn_name = build_fn_name(&derived.toolset.name, &tool.fn_name);
            let description = &tool.description;
            let params_json = tool
                .params
                .iter()
                .map(|param| {
                    let name = &param.name;
                    let json_type = &param.r#type;
                    let description = &param.description;

                    quote! {
                        #name: {
                            "type": #json_type,
                            "description": #description
                        }
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                ToolDefinition {
                    r#type: "function".to_string(),
                    name: #fn_name.to_string(),
                    description: #description.to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            #(#params_json),*
                        }
                    }),
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        fn definition(&self) -> Vec<ToolDefinition> {
            vec!(#(#definitions),*)
        }
    }
}

fn build_contain_fn(derived: &ToolsetDerive) -> proc_macro2::TokenStream {
    let fn_names = derived
        .toolset
        .tools
        .iter()
        .map(|tool| {
           build_fn_name(&derived.toolset.name, &tool.fn_name)
        })
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
        let fn_name = build_fn_name(&derived.toolset.name, &tool.fn_name);
        let fn_ident = syn::Ident::new(&tool.fn_name, proc_macro2::Span::call_site());

        if tool.params.is_empty() {
            return quote! {
                #fn_name => {
                    let result = self.#fn_ident().await?;
                    Ok(result)
                }
            };
        }
        let args_struct_ident = tool.args_struct_ident();

        quote! {
            #fn_name => {
                let args: #args_struct_ident = serde_json::from_str(args).map_err(|e| ToolError::WrongArguments(e))?;
                let result = self.#fn_ident(&args).await?;
                Ok(result)
            }
        }
  }).collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        async fn invoke( &self, fn_name: &str, args: &str) -> Result<ToolOutput, ToolError> {
            match fn_name {
                #(#invoke_matches)*
                _ => Err(ToolError::InvalidFunctionName(fn_name.to_string())),
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
    fn test_snapshot_derive_with_param() {
        let input: DeriveInput = parse_quote! {
            #[toolset(
                name = "Hello Derive",
                description="Hello derive",
                tool(
                    name = "hello",
                    description = "Hello world",
                    param(name = "arg1", r#type = "string"),
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
    fn test_snapshot_derive_with_multiple_params() {
        let input: DeriveInput = parse_quote! {
            #[toolset(
                name = "Hello Derive",
                description="Hello derive",
                tool(
                    name = "hello",
                    description = "Hello world",
                    param(name = "arg1", r#type = "string"), 
                    param(name = "arg2", r#type = "integer"),
                    param(name = "arg3", r#type = "boolean"),
                    param(name = "arg4", r#type = "array"),
                    param(name = "arg5", r#type = "object"),
                    param(name = "arg6", r#type = "null"),
                    param(name = "arg7", r#type = "any"),
                    param(name = "arg8", r#type = "number"),

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
    fn test_snapshot_derive_without_param() {
        let input: DeriveInput = parse_quote! {
            #[toolset(
                name = "Hello Derive",
                description="Hello derive",
                tool(
                    name = "hello",
                    description = "Hello world",
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
