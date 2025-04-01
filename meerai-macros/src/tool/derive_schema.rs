use quote::quote;
use syn::{Data, DeriveInput, Field, Fields};

pub fn schema_derive_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => generate_field_schemas(&fields.named),
            Fields::Unnamed(_) => panic!("Tuple structs are not supported"),
            Fields::Unit => Vec::new(),
        },
        Data::Enum(_) => {
            // Handle enums separately
            return Ok(handle_enum_derive(input));
        }
        Data::Union(_) => panic!("Unions are not supported"),
    };

    let expanded = quote! {
        impl meerai_core::JsonSchema for #name {
            fn json_schema() -> serde_json::Value {
                use serde_json::{Map, Value};

                let mut schema = Map::new();
                schema.insert("type".to_string(), Value::String("object".to_string()));

                let mut properties = Map::new();
                #(#fields)*

                schema.insert("properties".to_string(), Value::Object(properties));
                Value::Object(schema)
            }
        }
    };

    Ok(expanded)
}

fn handle_enum_derive(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;

    // This would extract enum variants and generate a schema
    // For demonstration, we'll just create a basic implementation
    let expanded = quote! {
        impl meerai_core::JsonSchema for #name {
            fn json_schema() -> serde_json::Value {
                use serde_json::{Map, Value};

                let mut schema = Map::new();
                schema.insert("type".to_string(), Value::String("string".to_string()));
                // In a real implementation, we would extract variant names here
                schema.insert("description".to_string(), Value::String(format!("Enum type {}", stringify!(#name))));

                Value::Object(schema)
            }
        }
    };

    expanded
}

fn generate_field_schemas(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> Vec<proc_macro2::TokenStream> {
    fields.iter().map(generate_field_schema).collect()
}

fn generate_field_schema(field: &Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;

    quote! {
        properties.insert(
            stringify!(#field_name).to_string(),
            <#field_type as meerai_core::JsonSchema>::json_schema()
        );
    }
}
