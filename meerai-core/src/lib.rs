pub mod chat_completion;
pub mod errors;
mod providers;
mod tools;

pub use async_trait::async_trait;
pub use providers::{
    gemini::{Gemini, GeminiConfig, Options as GeminiOptions},
    openrouter::{OpenRouter, OpenRouterConfig, Options as OpenRouterOptions},
};
pub use schemars::JsonSchema;
pub use tools::{ToolCall, ToolDefinition, ToolError, ToolOutput, Toolset};
