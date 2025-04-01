use meerai_core::JsonSchema;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, meerai_macros::Schema)]
pub struct GetWeatherArgs {
    arg1: String,
    arg2: Option<String>,
    arg3: f64,
    arg4: isize,
    arg5: Vec<String>,
}

#[futures_test::test]
async fn test_derive_schema_pass() {
    let args_schema = GetWeatherArgs::json_schema();
    println!("Args Schema: {}", args_schema);
}
