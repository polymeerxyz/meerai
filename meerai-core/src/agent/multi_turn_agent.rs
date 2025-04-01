use std::sync::Arc;

use crate::{
    ToolCall, ToolDefinition, ToolOutput, Toolset,
    chat_completion::{ChatCompletion, ChatCompletionRequest, ChatMessage},
};
use anyhow::{Error, Ok, Result};
use derive_builder::Builder;

const DEFAULT_SYSTEM_PROMPT: &str = "
You are a **Senior Assistant Specialist**.

Your role is to help the user successfully complete their request by utilizing all available tools at your disposal.

- Tasks may require multiple steps to complete.
- For each step, if a tool is needed, you should invoke it with the appropriate arguments.
- Once the tool responds, assess the result and determine the next appropriate step.
- Continue this process until the task is fully completed.
- When the task is complete, you must use the **special tool** called **stop_stop** to signal the end of the task.
";

#[derive(Debug, Builder)]
#[builder(build_fn(error = "Error"))]
pub struct MultiTurnAgent {
    chat_completion: Arc<dyn ChatCompletion>,
    #[builder(default = "Vec::new()")]
    chat_history: Vec<ChatMessage>,
    #[builder(default = "Vec::new()")]
    tools: Vec<Arc<dyn Toolset>>,
    #[builder(default = "DEFAULT_SYSTEM_PROMPT.to_string()")]
    system_prompt: String,
}

impl MultiTurnAgent {
    pub async fn prompt(&mut self, prompt: &str) -> Result<String> {
        let mut tool_definitions = Vec::<ToolDefinition>::new();
        for tool in &self.tools {
            tool_definitions.extend(tool.definition());
        }

        println!("Current Prompt: {:?}\n", prompt);

        self.chat_history
            .push(ChatMessage::System(self.system_prompt.clone()));
        self.chat_history
            .push(ChatMessage::User(prompt.to_string()));

        loop {
            let chat_completion_request = ChatCompletionRequest {
                model: None,
                messages: self.chat_history.clone(),
                tool_definitions: tool_definitions.clone(),
            };

            let chat_completion_response =
                self.chat_completion.send(&chat_completion_request).await?;
            self.chat_history
                .append(chat_completion_response.messages.clone().as_mut());

            if chat_completion_response.tool_calls.is_empty() {
                println!("Chat Completion: {:?}", chat_completion_response);
                return Ok(format!(
                    "Chat Completion: {:?}",
                    chat_completion_response
                        .messages
                        .last()
                        .unwrap_or(&ChatMessage::Assistant("No response".to_string()))
                ));
            }

            let ToolCall { name, args, .. } = chat_completion_response.tool_calls[0].clone();
            println!("Tool Call: {:?}", name);
            println!("Tool Arguments: {:?}", args);
            let tool = self
                .tools
                .iter()
                .find(|tool| tool.contain(&name))
                .ok_or_else(|| anyhow::anyhow!("Tool not found"))?;
            let tool_ouput = tool.invoke(&name, &args).await?;
            println!("Tool: {:?} args: {:?} output: {:?}", name, args, tool_ouput);

            if tool_ouput == ToolOutput::Stop {
                self.chat_history.clear();
                return Ok("Finished".to_string());
            }

            self.chat_history.push(ChatMessage::Assistant(format!(
                "Receive result: {} from tool: {}",
                tool_ouput, name,
            )));
        }
    }
}
