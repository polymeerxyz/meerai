use meerai_core::{JsonSchema, Toolset, async_trait};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, JsonSchema)]
pub struct GetWeatherArgs {
    pub location: String,
    pub unit: Option<String>,
    pub size: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, JsonSchema)]
pub struct SetWeatherArgs {
    pub location: String,
    pub unit: Option<String>,
    pub size: f64,
}

#[derive(meerai_macros::Toolset)]
#[toolset(
    tool(
        name = "Get Weather",
        description = "Get weather information",
        params = GetWeatherArgs,
    ),
    tool(
        name = "Set Weather",
        description = "Set weather information",
        params = SetWeatherArgs,
    )
)]
pub struct MyToolset;

#[async_trait]
impl MyToolsetInvoke for MyToolset {
    async fn get_weather(
        &self,
        args: &GetWeatherArgs,
    ) -> Result<meerai_core::ToolOutput, meerai_core::ToolError> {
        Ok(meerai_core::ToolOutput::Text(args.location.clone()))
    }

    async fn set_weather(
        &self,
        args: &SetWeatherArgs,
    ) -> Result<meerai_core::ToolOutput, meerai_core::ToolError> {
        Ok(meerai_core::ToolOutput::Text(args.location.clone()))
    }
}

#[futures_test::test]
async fn test_derive_toolset_pass() {
    let tool = MyToolset {};
    assert_eq!(tool.name(), "my_toolset");

    let args = GetWeatherArgs {
        location: "Menlo Park, CA".to_string(),
        unit: Some("C".to_string()),
        size: 1.0f64,
    };
    let direct_call = tool.get_weather(&args).await.unwrap();

    let json_args = "{ \"location\": \"Menlo Park, CA\", \"size\": 1.0 }";
    let invoke_call = tool
        .invoke("my_toolset-get_weather", json_args)
        .await
        .unwrap();

    let definition = tool.definition();
    println!("{}", serde_json::to_string_pretty(&definition).unwrap());
    assert_eq!(direct_call, invoke_call);
}
