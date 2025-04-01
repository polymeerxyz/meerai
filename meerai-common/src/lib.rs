use anyhow::Result;
use meerai_macros::Toolset;

#[derive(Toolset)]
#[toolset(
    name = "stop",
    description = "Stop the meerai agent.",
    tool(name = "stop", description = "Stop the meerai agent.")
)]
pub struct Stop;

#[async_trait::async_trait]
impl Invoke for Stop {
    async fn stop(&self) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput::Stop)
    }
}
