use anyhow::Result;
use meerai_macros::{Schema, tool};

#[tool(name = "Stop", description = "Stop the agent.")]
async fn stop() -> Result<meerai_core::ToolOutput, meerai_core::ToolError> {
    Ok(meerai_core::ToolOutput::Stop("".to_string()))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Schema)]
pub struct StopWithReasonArgs {
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use meerai_core::{ToolOutput, Toolset};

    #[futures_test::test]
    async fn test_stop_tool() {
        let toolset = StopToolset;
        let definitions = toolset.definition();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "stop-stop");

        let result = toolset.stop().await.unwrap();
        assert_eq!(result, ToolOutput::Stop("".to_string()));
    }
}
