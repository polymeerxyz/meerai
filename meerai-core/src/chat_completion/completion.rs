use std::fmt::Debug;

use anyhow::Result;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestDeveloperMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ToolCall, ToolDefinition};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChatMessage {
    Assistant(String),
    Developer(String),
    System(String),
    User(String),
}

#[derive(Clone, Debug)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub tool_definitions: Vec<ToolDefinition>,
}

#[derive(Clone, Debug)]
pub struct ChatCompletionResponse {
    pub messages: Vec<ChatMessage>,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Error, Debug)]
pub enum ChatCompletionError {
    #[error("unexpected error: {0}")]
    Literal(String),

    #[error("llm returned an error: {0}")]
    LLM(#[from] async_openai::error::OpenAIError),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait ChatCompletion {
    async fn send(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ChatCompletionError>;
}

impl Debug for dyn ChatCompletion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ChatCompletion")
    }
}

pub fn message_to_openai(message: &ChatMessage) -> Result<ChatCompletionRequestMessage> {
    let openai_message: ChatCompletionRequestMessage = match message {
        ChatMessage::Assistant(content) => ChatCompletionRequestAssistantMessageArgs::default()
            .content(content.clone())
            .build()?
            .into(),
        ChatMessage::Developer(content) => ChatCompletionRequestDeveloperMessageArgs::default()
            .content(content.clone())
            .build()?
            .into(),
        ChatMessage::System(content) => ChatCompletionRequestSystemMessageArgs::default()
            .content(content.clone())
            .build()?
            .into(),
        ChatMessage::User(content) => ChatCompletionRequestUserMessageArgs::default()
            .content(content.clone())
            .build()?
            .into(),
    };

    Ok(openai_message)
}
