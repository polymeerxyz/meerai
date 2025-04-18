use std::time::Duration;
use std::{pin::Pin, vec};

use anyhow::{Result, anyhow};
use meerai_core::{
    ToolCall, ToolDefinition, ToolOutput, Toolset,
    chat_completion::{ChatCompletion, ChatCompletionRequest, ChatMessage},
};

/// Configuration for the MultiTurnAgent.
#[derive(Debug, Clone)]
pub struct MultiTurnAgentConfig {
    /// Maximum number of cycles the agent will run in a single prompt
    pub max_cycles: usize,

    /// Maximum number of times to retry a failed tool invocation
    pub max_retries: usize,
}

impl Default for MultiTurnAgentConfig {
    fn default() -> Self {
        Self {
            max_cycles: 10,
            max_retries: 3,
        }
    }
}

/// A multi-turn agent that can interact with tools and maintain conversation history.
///
/// This agent handles:
/// - Managing conversation history
/// - Executing tool calls
/// - Processing system and user prompts
#[derive(Debug)]
pub struct MultiTurnAgent {
    /// The underlying chat completion provider
    chat_completion: Pin<Box<dyn ChatCompletion>>,

    /// History of the conversation
    chat_history: Vec<ChatMessage>,

    /// Available tools for the agent to use
    tools: Vec<Pin<Box<dyn Toolset>>>,

    /// System prompt that guides the agent's behavior
    system_prompt: String,

    /// Configuration for the agent
    config: MultiTurnAgentConfig,
}

impl MultiTurnAgent {
    /// Creates a new MultiTurnAgent with the specified chat completion provider, tools, and system prompt.
    ///
    /// # Arguments
    ///
    /// * `chat_completion` - The chat completion provider to use
    /// * `tools` - A vector of tools available to the agent
    /// * `system_prompt` - The system prompt that guides the agent's behavior
    pub fn new(
        chat_completion: impl ChatCompletion + 'static,
        tools: Vec<Pin<Box<dyn Toolset>>>,
        system_prompt: String,
    ) -> Self {
        Self {
            chat_completion: Box::pin(chat_completion),
            chat_history: vec![],
            tools,
            system_prompt,
            config: MultiTurnAgentConfig::default(),
        }
    }

    /// Creates a new MultiTurnAgent with custom configuration
    pub fn new_with_config(
        chat_completion: impl ChatCompletion + 'static,
        tools: Vec<Pin<Box<dyn Toolset>>>,
        system_prompt: String,
        config: MultiTurnAgentConfig,
    ) -> Self {
        Self {
            chat_completion: Box::pin(chat_completion),
            chat_history: vec![],
            tools,
            system_prompt,
            config,
        }
    }

    /// Creates a new MultiTurnAgent without any tools.
    ///
    /// # Arguments
    ///
    /// * `chat_completion` - The chat completion provider to use
    /// * `system_prompt` - The system prompt that guides the agent's behavior
    pub fn new_without_tools(
        chat_completion: impl ChatCompletion + 'static,
        system_prompt: String,
    ) -> Self {
        Self::new(chat_completion, vec![], system_prompt)
    }

    /// Adds a single tool to the agent.
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool to add
    pub fn add_tool(&mut self, tool: impl Toolset + 'static) {
        self.tools.push(Box::pin(tool));
    }

    /// Adds multiple tools to the agent.
    ///
    /// # Arguments
    ///
    /// * `tools` - A vector of tools to add
    pub fn add_tools(&mut self, tools: Vec<Pin<Box<dyn Toolset>>>) {
        self.tools.extend(tools);
    }

    /// Sends a prompt to the agent and processes the response.
    ///
    /// This method:
    /// 1. Collects tool definitions
    /// 2. Adds the system and user prompts to the chat history
    /// 3. Sends the request to the chat completion provider
    /// 4. Processes any tool calls
    /// 5. Returns the final response
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user prompt to send
    ///
    /// # Returns
    ///
    /// A Result containing the agent's response as a String
    pub async fn prompt(&mut self, prompt: &str) -> Result<String> {
        let mut tool_definitions = Vec::<ToolDefinition>::new();
        for tool in &self.tools {
            tool_definitions.extend(tool.definition());
        }

        println!("[Agent] Current Prompt: {:?}", prompt);
        self.chat_history.clear();
        self.chat_history
            .push(ChatMessage::System(self.system_prompt.clone()));
        self.chat_history
            .push(ChatMessage::User(prompt.to_string()));

        let mut cycle_count = 0;

        loop {
            // Check if we've exceeded the maximum number of cycles
            if cycle_count >= self.config.max_cycles {
                return Err(anyhow!(
                    "Exceeded maximum number of cycles ({})",
                    self.config.max_cycles
                ));
            }

            cycle_count += 1;
            let chat_completion_request = ChatCompletionRequest {
                model: None,
                messages: self.chat_history.clone(),
                tool_definitions: tool_definitions.clone(),
            };

            let chat_completion_response =
                self.chat_completion.send(&chat_completion_request).await?;
            self.chat_history
                .append(chat_completion_response.messages.clone().as_mut());

            // If no tool calls, return the final response
            if chat_completion_response.tool_calls.is_empty() {
                println!("[Agent] Chat Completion: {:?}", chat_completion_response);

                let last_message = match chat_completion_response.messages.last() {
                    Some(msg) => format!("{:?}", msg),
                    None => "No response".to_string(),
                };

                return Ok(format!("Chat Completion: {}", last_message));
            }

            let ToolCall { name, args, .. } = chat_completion_response.tool_calls[0].clone();
            println!("[Tool] Invoking: '{}' with args: {:?}", name, args);

            let tool = self
                .tools
                .iter()
                .find(|tool| tool.contain(&name))
                .ok_or_else(|| anyhow!("Tool not found: {}", name))?;

            // Handle tool invocation with retry logic
            let tool_output = self.invoke_tool_with_retry(tool, &name, &args).await?;
            println!("[Tool] Result: {:?}", tool_output);

            if let ToolOutput::Stop(_) = tool_output {
                self.chat_history.clear();
                return Ok("Finished".to_string());
            }

            self.chat_history.push(ChatMessage::Assistant(format!(
                "Receive result: {} from tool: {}",
                tool_output, name,
            )));
        }
    }

    /// Invokes a tool with retry logic
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

                    if retry_count >= self.config.max_retries {
                        let error_message = format!(
                            "[Tool Error] Failed to invoke tool '{}' after {} retries: {}",
                            name, self.config.max_retries, err
                        );
                        println!("{}", error_message);
                        return Err(anyhow!(error_message));
                    }

                    // Wait briefly before retrying
                    std::thread::sleep(Duration::from_millis(500));
                    println!(
                        "[Tool] Retry {}/{} for '{}': {}",
                        retry_count, self.config.max_retries, name, err
                    );
                }
            }
        }
    }
}
