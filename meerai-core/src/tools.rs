use std::fmt::Debug;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::async_trait;

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub output: String,
}

impl std::fmt::Display for CommandOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.output)
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("executor error: {0:#}")]
    ExecutorError(#[from] anyhow::Error),

    #[error("command failed with NonZeroExit: {0}")]
    NonZeroExit(CommandOutput),
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("invalid function name: {0:#}")]
    InvalidFunctionName(String),

    #[error("arguments for tool failed to parse: {0:#}")]
    WrongArguments(#[from] serde_json::Error),

    #[error("tool execution failed: {0:#}")]
    ExecutionFailed(#[from] CommandError),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ToolOutput {
    Text(String),

    Fail(String),

    Stop(String),
}

impl std::fmt::Display for ToolOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolOutput::Text(text) => write!(f, "Success: {}", text),
            ToolOutput::Fail(text) => write!(f, "Failure: {}", text),
            ToolOutput::Stop(text) => write!(f, "Stop: {}", text),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolDefinition {
    pub r#type: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    pub fn to_openai(&self) -> Result<async_openai::types::ChatCompletionTool> {
        let function = serde_json::to_string(self)?;
        let function_object: async_openai::types::FunctionObject = serde_json::from_str(&function)?;
        let chat_completion_tool = async_openai::types::ChatCompletionToolArgs::default()
            .r#type(async_openai::types::ChatCompletionToolType::Function)
            .function(function_object)
            .build()?;
        Ok(chat_completion_tool)
    }
}

#[async_trait]
pub trait Toolset {
    fn name(&self) -> String;

    fn definition(&self) -> Vec<ToolDefinition>;

    fn contain(&self, fn_name: &str) -> bool;

    async fn invoke(&self, fn_name: &str, args: &str) -> Result<ToolOutput, ToolError>;
}

impl Debug for dyn Toolset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Toolset {}", self.name())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ToolCall {
    // pub id: String,
    pub name: String,
    pub args: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[test]
    fn test_tool() {
        struct MyToolset;

        #[async_trait]
        impl Toolset for MyToolset {
            fn name(&self) -> String {
                "my_tool".to_string()
            }

            fn definition(&self) -> Vec<ToolDefinition> {
                vec![ToolDefinition {
                    r#type: "function".to_string(),
                    name: self.name(),
                    description: "My tool".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "arg1": {
                                "type": "string",
                                "description": "Argument 1"
                            }
                        }
                    }),
                }]
            }

            fn contain(&self, fn_name: &str) -> bool {
                self.name() == fn_name
            }

            async fn invoke(&self, _fn_name: &str, _args: &str) -> Result<ToolOutput, ToolError> {
                Ok(ToolOutput::Text("Hello world".to_string()))
            }
        }

        assert!(true);
    }
}
