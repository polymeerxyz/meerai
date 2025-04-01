use anyhow::{Context, Result};
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role};
use derive_builder::Builder;
use reqwest::header::HeaderMap;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    ToolCall,
    chat_completion::{
        ChatCompletion, ChatCompletionError, ChatCompletionRequest, ChatCompletionResponse,
        ChatMessage, message_to_openai,
    },
};

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";

#[derive(Clone, Debug, Deserialize)]
pub struct GeminiConfig {
    api_url: String,
    api_key: SecretString,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_url: GEMINI_API_URL.to_string(),
            api_key: std::env::var("GEMINI_API_KEY")
                .unwrap_or_else(|_| String::new())
                .into(),
        }
    }
}

impl async_openai::config::Config for GeminiConfig {
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let api_key = self.api_key.expose_secret();
        assert!(!api_key.is_empty(), "API key for Gemini is required");

        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );

        headers
    }

    fn api_base(&self) -> &str {
        &self.api_url
    }

    fn api_key(&self) -> &SecretString {
        &self.api_key
    }

    fn query(&self) -> Vec<(&str, &str)> {
        vec![]
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_url, path)
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub prompt_model: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            prompt_model: "gemini-1.5-flash".to_string(),
        }
    }
}

#[derive(Debug, Builder)]
pub struct Gemini {
    #[builder(default = "Arc::new(async_openai::Client::with_config(GeminiConfig::default()))")]
    pub client: Arc<async_openai::Client<GeminiConfig>>,
    #[builder(default = "Options::default()")]
    pub default_options: Options,
}

impl Gemini {
    pub fn new(api_url: &str, api_key: &str) -> Self {
        Self {
            client: Arc::new(async_openai::Client::with_config(GeminiConfig {
                api_url: api_url.to_string(),
                api_key: api_key.to_string().into(),
            })),
            default_options: Options::default(),
        }
    }

    pub fn new_with_options(api_url: &str, api_key: &str, options: Options) -> Self {
        Self {
            client: Arc::new(async_openai::Client::with_config(GeminiConfig {
                api_url: api_url.to_string(),
                api_key: api_key.to_string().into(),
            })),
            default_options: options,
        }
    }

    pub fn set_default_options(&mut self, options: Options) {
        self.default_options = options;
    }
}

#[async_trait::async_trait]
impl ChatCompletion for Gemini {
    async fn send(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ChatCompletionError> {
        let model = request
            .model
            .clone()
            .unwrap_or(self.default_options.prompt_model.clone());

        let messages: Vec<ChatCompletionRequestMessage> = request
            .messages
            .iter()
            .map(message_to_openai)
            .collect::<Result<Vec<_>>>()
            .context("Failed to convert messages")?;

        let mut openai_request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .to_owned();

        let tools = if request.tool_definitions.is_empty() {
            vec![]
        } else {
            request
                .tool_definitions
                .iter()
                .map(|tool| tool.to_openai())
                .collect::<Result<Vec<_>>>()
                .context("Failed to convert tools")?
        };

        let req = openai_request.tool_choice("auto").tools(tools).build()?;
        let res = self.client.chat().create(req).await?;

        let chat_completion_response: ChatCompletionResponse = ChatCompletionResponse {
            messages: res
                .choices
                .iter()
                .filter_map(|choice| {
                    choice
                        .message
                        .content
                        .as_ref()
                        .and_then(|content| match choice.message.role {
                            Role::Assistant => Some(ChatMessage::Assistant(content.to_string())),
                            Role::System => Some(ChatMessage::System(content.to_string())),
                            Role::User => Some(ChatMessage::User(content.to_string())),
                            _ => None,
                        })
                })
                .collect(),
            tool_calls: res
                .choices
                .first()
                .and_then(|choice| choice.message.tool_calls.clone())
                .unwrap_or_default()
                .iter()
                .map(|tool_call| ToolCall {
                    // id: tool_call.id,
                    name: tool_call.function.name.clone(),
                    args: tool_call.function.arguments.clone(),
                })
                .collect::<Vec<ToolCall>>(),
        };

        Ok(chat_completion_response)
    }
}
