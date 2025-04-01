pub fn pretty_macro_output(item: &proc_macro2::TokenStream) -> String {
    let file = syn::parse_file(&item.to_string())
        .unwrap_or_else(|_| panic!("Failed to parse token stream: {}", &item.to_string()));
    prettyplease::unparse(&file)
}
