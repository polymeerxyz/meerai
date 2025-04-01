pub mod chat_completion;
pub mod errors;
pub mod json_schema;
mod providers;
mod tools;

pub use async_trait::async_trait;
pub use json_schema::JsonSchema;
pub use providers::{
    gemini::{GeminiBuilder, GeminiConfig, Options as GeminiOptions},
    openrouter::{OpenRouterBuilder, OpenRouterConfig, Options as OpenRouterOptions},
};
pub use tools::{ToolCall, ToolDefinition, ToolError, ToolOutput, Toolset};
