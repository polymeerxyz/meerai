use std::{pin::Pin, time::Duration};

use anyhow::{Result, anyhow};
use bsky_sdk::BskyAgent;
use meerai_core::{
    ToolCall, ToolOutput, Toolset,
    chat_completion::{ChatCompletion, ChatCompletionRequest, ChatMessage},
};
use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};

use crate::{config::BlueskyConfig, tools};

const DEFAULT_SYSTEM_PROMPT: &str = "
You are a **Bluesky Social Media Assistant**.

Your role is to help manage and interact with the Bluesky social network by:
- Composing and posting thoughtful content
- Engaging with other users' posts
- Managing your account and notifications
- Using all available tools to complete tasks

Guidelines:
- Be professional and thoughtful in all interactions
- Tasks may require multiple steps - plan carefully
- Always verify tool results before proceeding
- Use the stop tool when the task is complete
";

/// Configuration for the BlueskyActor
#[derive(Debug, Clone)]
pub struct BlueskyAgentConfig {
    /// Maximum number of cycles the agent will run in a single prompt
    pub max_cycles: usize,

    /// Maximum number of times to retry a failed tool invocation
    pub max_retries: usize,
}

impl BlueskyActor {
    async fn process_prompt(
        &self,
        prompt: &str,
        chat_history: &mut Vec<ChatMessage>,
    ) -> Result<(), anyhow::Error> {
        let mut tool_definitions = Vec::new();
        for tool in &self.tools {
            tool_definitions.extend(tool.definition());
        }

        chat_history.clear();
        chat_history.push(ChatMessage::System(DEFAULT_SYSTEM_PROMPT.to_string()));
        chat_history.push(ChatMessage::User(prompt.to_string()));

        let mut cycle_count = 0;

        loop {
            if cycle_count >= self.agent_config.max_cycles {
                return Err(anyhow!(
                    "Exceeded maximum number of cycles ({})",
                    self.agent_config.max_cycles
                ));
            }

            cycle_count += 1;
            let request = ChatCompletionRequest {
                model: None,
                messages: chat_history.clone(),
                tool_definitions: tool_definitions.clone(),
            };

            let response = self.chat_completion.send(&request).await?;
            chat_history.extend(response.messages);

            if response.tool_calls.is_empty() {
                break;
            }

            for tool_call in response.tool_calls {
                let ToolCall { name, args, .. } = tool_call;
                let tool = self
                    .tools
                    .iter()
                    .find(|t| t.contain(&name))
                    .ok_or_else(|| anyhow!("Tool not found: {}", name))?;

                let output = self.invoke_tool_with_retry(tool, &name, &args).await?;
                chat_history.push(ChatMessage::Assistant(format!(
                    "Tool {} result: {}",
                    name, output
                )));

                if let ToolOutput::Stop(_) = output {
                    chat_history.clear();
                    break;
                }
            }
        }

        Ok(())
    }

    async fn invoke_tool_with_retry(
        &self,
        tool: &Pin<Box<dyn Toolset>>,
        name: &str,
        args: &str,
    ) -> Result<ToolOutput> {
        let mut retry_count = 0;

        loop {
            match tool.invoke(name, args).await {
                Ok(output) => return Ok(output),
                Err(err) => {
                    retry_count += 1;
                    if retry_count >= self.agent_config.max_retries {
                        return Err(anyhow!(
                            "Failed to invoke tool '{}' after {} retries: {}",
                            name,
                            self.agent_config.max_retries,
                            err
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
}

impl Default for BlueskyAgentConfig {
    fn default() -> Self {
        Self {
            max_cycles: 10,
            max_retries: 3,
        }
    }
}

pub struct BlueskyActor {
    /// Configuration for the Bluesky tool
    config: BlueskyConfig,

    /// The underlying chat completion provider
    chat_completion: Pin<Box<dyn ChatCompletion>>,

    /// Available tools for the agent to use
    tools: Vec<Pin<Box<dyn Toolset>>>,

    /// Agent configuration
    agent_config: BlueskyAgentConfig,
}

impl BlueskyActor {
    pub async fn new(
        config: BlueskyConfig,
        chat_completion: impl ChatCompletion + 'static,
    ) -> Result<Self, anyhow::Error> {
        let bsky_agent = BskyAgent::builder().build().await?;
        bsky_agent
            .login(&config.identifier, &config.password)
            .await?;

        Ok(Self {
            config,
            chat_completion: Box::pin(chat_completion),
            tools: vec![Box::pin(tools::BskyToolset::new(bsky_agent.clone()))],
            agent_config: BlueskyAgentConfig::default(),
        })
    }

    pub fn add_tool(&mut self, tool: impl Toolset + 'static) {
        self.tools.push(Box::pin(tool));
    }

    pub fn add_tools(&mut self, tools: Vec<Pin<Box<dyn Toolset>>>) {
        self.tools.extend(tools);
    }
}

#[derive(Debug, Clone)]
pub enum BlueskyMessage {
    Prompt(String),
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
enum Status {
    Idle,
    Working,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct BlueskyState {
    status: Status,

    /// History of the conversation
    chat_history: Vec<ChatMessage>,
}

#[async_trait]
impl Actor for BlueskyActor {
    type Msg = BlueskyMessage;
    type Arguments = ();
    type State = BlueskyState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::State {
            status: Status::Idle,
            chat_history: vec![],
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            BlueskyMessage::Prompt(prompt) => match state.status {
                Status::Idle => {
                    state.status = Status::Working;
                    if let Err(e) = self.process_prompt(&prompt, &mut state.chat_history).await {
                        eprintln!("Error processing prompt: {}", e);
                        state.status = Status::Idle;
                    } else {
                        state.status = Status::Idle;
                    }
                    state.chat_history.clear();
                }
                Status::Working => {
                    eprintln!("Actor is already working");
                }
                Status::Stopped => {
                    eprintln!("Actor is stopped");
                }
            },
            BlueskyMessage::Stop => {
                state.status = Status::Stopped;
                myself.stop(Some("Stopped by user".to_string()));
            }
        }
        Ok(())
    }
}
