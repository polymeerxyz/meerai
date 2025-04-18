use meerai_core::JsonSchema;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, JsonSchema)]
pub struct ChildrenArgs {
    chil: String,
    arg2: Option<String>,
    arg3: f64,
    arg4: isize,
    arg5: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, JsonSchema)]
pub struct SampleArgs {
    arg1: String,
    arg2: Option<String>,
    arg3: f64,
    arg4: isize,
    arg5: Vec<String>,
    arg6: ChildrenArgs,
}

#[futures_test::test]
async fn test_derive_schema_pass() {
    let generator = &mut schemars::SchemaGenerator::new(
        schemars::generate::SchemaSettings::default().with(|s| {
            s.meta_schema = None;
        }),
    );
    let value: serde_json::Value = SampleArgs::json_schema(generator).into();
    println!("Args Schema: {}", value);
}
