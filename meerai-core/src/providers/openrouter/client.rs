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

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1";

#[derive(Clone, Debug, Deserialize)]
pub struct OpenRouterConfig {
    api_url: String,
    api_key: SecretString,
    site_url: Option<String>,
    site_name: Option<String>,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_url: OPENROUTER_API_URL.to_string(),
            api_key: std::env::var("OPENROUTER_API_KEY")
                .unwrap_or_else(|_| String::new())
                .into(),
            site_url: None,
            site_name: None,
        }
    }
}

impl async_openai::config::Config for OpenRouterConfig {
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let api_key = self.api_key.expose_secret();
        assert!(!api_key.is_empty(), "API key for OpenRouter is required");

        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );

        // add custom headers if site_url is set and site_name is set
        // site_url is used as HTTP-Referer
        // site_name is used as X-Title
        // [OpenRouter API Reference](https://openrouter.ai/docs/api-reference/overview#headers)
        if let Some(site_url) = &self.site_url {
            headers.insert("HTTP-Referer", site_url.parse().unwrap());
        }

        if let Some(site_name) = &self.site_name {
            headers.insert("X-Title", site_name.parse().unwrap());
        }
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
            prompt_model: "meta-llama/llama-4-maverick".to_string(),
        }
    }
}

#[derive(Debug, Builder)]
pub struct OpenRouter {
    #[builder(
        default = "Arc::new(async_openai::Client::with_config(OpenRouterConfig::default()))"
    )]
    pub client: Arc<async_openai::Client<OpenRouterConfig>>,
    #[builder(default = "Options::default()")]
    pub default_options: Options,
}

#[async_trait::async_trait]
impl ChatCompletion for OpenRouter {
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
                    // id: tool_call.id.clone(),
                    name: tool_call.function.name.clone(),
                    args: tool_call.function.arguments.clone(),
                })
                .collect::<Vec<ToolCall>>(),
        };

        Ok(chat_completion_response)
    }
}
