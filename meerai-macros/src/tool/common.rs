use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromMeta};

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct ToolMeta {
    pub name: String,

    pub description: String,
}

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(toolset),
    supports(struct_any),
    forward_attrs(allow, doc, cfg)
)]
pub struct ToolsetDerive {
    pub ident: syn::Ident,

    #[allow(dead_code)]
    pub attrs: Vec<syn::Attribute>,

    #[darling(flatten)]
    pub toolset: ToolsetArgs,
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct ToolsetArgs {
    pub name: String,

    #[darling(multiple, rename = "tool")]
    pub tools: Vec<ToolArgs>,
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct ToolArgs {
    pub name: String,

    pub description: String,

    pub params: Option<syn::Path>,
}

impl ToolArgs {
    pub fn get_fn_name(&self) -> String {
        self.name.to_case(Case::Snake)
    }

    pub fn args_struct_ident(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("{}Args", &self.get_fn_name().to_case(Case::Pascal)),
            proc_macro2::Span::call_site(),
        )
    }
}
