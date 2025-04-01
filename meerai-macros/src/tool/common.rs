use convert_case::{Case, Casing};
use darling::{Error, FromDeriveInput, FromMeta};
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(toolset), supports(struct_any), and_then = ToolsetDerive::update_defaults, forward_attrs(allow, doc, cfg))]
pub struct ToolsetDerive {
    pub ident: syn::Ident,

    #[allow(dead_code)]
    pub attrs: Vec<syn::Attribute>,

    #[darling(flatten)]
    pub toolset: ToolsetArgs,
}

impl ToolsetDerive {
    pub fn update_defaults(mut self) -> Result<Self, Error> {
        for tool in &mut self.toolset.tools {
            tool.infer_fn_name();
            tool.infer_param_types()?;
        }
        Ok(self)
    }
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct ToolsetArgs {
    pub name: String,

    pub description: String,

    #[darling(multiple, rename = "tool")]
    pub tools: Vec<ToolArgs>,
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct ToolArgs {
    pub name: String,

    pub fn_name: String,

    pub description: String,

    #[darling(multiple, rename = "param")]
    pub params: Vec<Param>,
}

impl ToolArgs {
    pub fn infer_fn_name(&mut self) {
        if self.fn_name.is_empty() {
            self.fn_name = self.name.to_case(Case::Snake);
        }
    }

    pub fn infer_param_types(&mut self) -> Result<(), Error> {
        for param in &mut self.params {
            param.rust_type = match param.r#type.as_str() {
                "array" => syn::parse_quote! { Vec<String> },
                "boolean" => syn::parse_quote! { bool },
                "null" => syn::parse_quote! { () },
                "integer" => syn::parse_quote! { isize },
                "number" => syn::parse_quote! { f64 },
                "object" => syn::parse_quote! { serde_json::Value },
                "string" => syn::parse_quote! { String },
                _ => syn::parse_quote! { serde_json::Value },
            };
        }
        Ok(())
    }

    pub fn args_struct_ident(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("{}Args", &self.fn_name.to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        )
    }

    pub fn args_struct(&self) -> proc_macro2::TokenStream {
        if self.params.is_empty() {
            return quote! {};
        }

        let mut fields = vec![];

        for param in &self.params {
            let ty = &param.rust_type;
            let ident = syn::Ident::new(&param.name, proc_macro2::Span::call_site());
            if param.required {
                fields.push(quote! { pub #ident: #ty });
            } else {
                fields.push(quote! { pub #ident: Option<#ty> });
            }
        }

        let args_struct_ident = self.args_struct_ident();

        quote! {
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            pub struct #args_struct_ident {
                // hello world
                #(#fields),*
            }

            impl std::fmt::Display for #args_struct_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", serde_json::to_string(self).unwrap())
                }
            }
        }
    }
}

#[derive(Debug, FromMeta)]
#[darling(default)]
pub struct Param {
    pub name: String,

    pub description: String,

    pub r#type: String,

    #[darling(skip = true)]
    pub rust_type: syn::Type,

    pub required: bool,
}

impl Default for Param {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            r#type: String::new(),
            rust_type: syn::parse_quote! { String },
            required: true,
        }
    }
}
