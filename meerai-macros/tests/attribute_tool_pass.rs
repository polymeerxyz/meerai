use meerai_core::{JsonSchema, Toolset};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, JsonSchema)]
pub struct GetWeatherArgs {
    location: String,
    unit: Option<String>,
    size: f64,
}

#[meerai_macros::tool(name = "Get Weather", description = "Get weather information")]
async fn get_weather(
    args: &GetWeatherArgs,
) -> Result<meerai_core::ToolOutput, meerai_core::ToolError> {
    Ok(meerai_core::ToolOutput::Text(args.location.clone()))
}

#[futures_test::test]
async fn test_attribute_tool_pass() {
    let tool = GetWeatherToolset;

    let args = GetWeatherArgs {
        location: "Menlo Park, CA".to_string(),
        unit: Some("C".to_string()),
        size: 1.0f64,
    };
    let direct_call = tool.get_weather(&args).await.unwrap();

    let json_args = "{ \"location\": \"Menlo Park, CA\", \"size\": 1.0 }";
    let invoke_call = tool
        .invoke("get_weather-get_weather", json_args)
        .await
        .unwrap();

    let definition = tool.definition();
    println!("{}", serde_json::to_string_pretty(&definition).unwrap());
    assert_eq!(direct_call, invoke_call);
}
