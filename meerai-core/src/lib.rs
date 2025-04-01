pub mod agent;
pub mod chat_completion;
pub mod errors;
mod providers;
mod tools;

pub use providers::{
    gemini::{GeminiBuilder, GeminiConfig, Options as GeminiOptions},
    openrouter::{OpenRouterBuilder, OpenRouterConfig, Options as OpenRouterOptions},
};
pub use tools::{Tool, ToolCall, ToolDefinition, ToolError, ToolOutput, Toolset};
