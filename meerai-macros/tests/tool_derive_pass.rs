#[allow(dead_code)]
use meerai_macros::Toolset;

#[derive(Toolset)]
#[toolset(
    name = "My Toolset",
    description = "A set of tools for various tasks",
    tool(
        name = "Get Weather",
        description = "Get weather information",
        param(name = "location", r#type = "string"),
        param(name = "unit", r#type = "string", required = false),
        param(name = "size", r#type = "number"),
    ),
    tool(
        name = "Set Weather",
        description = "Set weather information",
        param(name = "location", r#type = "string"),
        param(name = "unit", r#type = "string", required = false),
        param(name = "size", r#type = "number"),
    )
)]
pub struct MyToolset;

#[async_trait::async_trait]
impl Invoke for MyToolset {
    async fn get_weather(&self, args: &GetWeatherArgs) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput::Text(args.location.clone()))
    }

    async fn set_weather(&self, args: &SetWeatherArgs) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput::Text(args.location.clone()))
    }
}

#[futures_test::test]
async fn test_tool_derive_pass() {
    let tool = MyToolset {};
    let args = GetWeatherArgs {
        location: "Menlo Park, CA".to_string(),
        unit: Some("C".to_string()),
        size: 1.0f64,
    };
    let json_args = "{ \"location\": \"Menlo Park, CA\", \"size\": 1.0 }";

    let direct_call = tool.get_weather(&args).await.unwrap();
    let invoke_call = tool
        .invoke("my_toolset-get_weather", json_args)
        .await
        .unwrap();

    let definition = tool.definition();
    println!("{}", serde_json::to_string_pretty(&definition).unwrap());
    assert_eq!(direct_call, invoke_call);
}
