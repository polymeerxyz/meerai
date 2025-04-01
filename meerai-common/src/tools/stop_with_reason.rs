use anyhow::Result;
use meerai_core::JsonSchema;
use meerai_macros::{Schema, tool};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Schema)]
pub struct StopWithReasonArgs {
    pub reason: String,
}

#[tool(
    name = "Stop with Reason",
    description = "Stop the agent with a reason."
)]
async fn stop_with_reason(
    args: &StopWithReasonArgs,
) -> Result<meerai_core::ToolOutput, meerai_core::ToolError> {
    Ok(meerai_core::ToolOutput::Stop(args.reason.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use meerai_core::{ToolOutput, Toolset};

    #[futures_test::test]
    async fn test_stop_with_reason_tool() {
        let toolset = StopWithReasonToolset;
        let definitions = toolset.definition();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "stop_with_reason-stop_with_reason");

        let args = StopWithReasonArgs {
            reason: "Test reason".to_string(),
        };
        let result = toolset.stop_with_reason(&args).await.unwrap();
        assert_eq!(result, ToolOutput::Stop("Test reason".to_string()));
    }
}
