use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use rig::{
    OneOrMany,
    agent::Agent,
    completion::{Completion, CompletionModel},
    message::{AssistantContent, Message, ToolResultContent, UserContent},
};

pub struct Session<T: CompletionModel, F: Fn(&str) -> ()> {
    agent: Agent<T>,
    history: Vec<Message>,
    on_message: F,
}

impl<T: CompletionModel, F: Fn(&str) -> ()> Session<T, F> {
    pub fn new(agent: Agent<T>, on_message: F) -> Self {
        Self {
            agent,
            history: Vec::new(),
            on_message,
        }
    }

    fn create_spinner(&self, message: &str) -> ProgressBar {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner
    }

    pub async fn message(
        &mut self,
        prompt: impl Into<Message> + Send + Clone,
    ) -> anyhow::Result<()> {
        let mut tool_results = Vec::new();
        let mut tool_calls = Vec::new();
        let mut did_call_tool = false;
        let mut current_prompt: Message = prompt.clone().into();
        let mut current_tool_call: Option<String> = None;

        loop {
            let spin_message = if current_tool_call.is_some() {
                format!("Working... ({})", current_tool_call.as_ref().unwrap(),)
            } else {
                "Working...".to_string()
            };
            let spinner = self.create_spinner(&spin_message);
            spinner.enable_steady_tick(Duration::from_millis(100));
            let request = self
                .agent
                .completion(current_prompt.clone(), self.history.clone())
                .await?
                .build();
            let result = self.agent.model.completion(request).await?;
            spinner.finish_and_clear();

            self.history.push(current_prompt.clone());

            for choice in result.choice {
                match choice {
                    rig::message::AssistantContent::Text(text) => {
                        self.history.push(Message::Assistant {
                            content: OneOrMany::one(AssistantContent::text(text.text.clone())),
                        });
                        (self.on_message)(&text.text);
                    }
                    rig::message::AssistantContent::ToolCall(tool_call) => {
                        let tool_result = self
                            .agent
                            .tools
                            .call(
                                &tool_call.function.name,
                                tool_call.function.arguments.to_string(),
                            )
                            .await?;

                        tool_results.push((tool_call.id.clone(), tool_result.clone()));
                        tool_calls.push(AssistantContent::tool_call(
                            tool_call.id.clone(),
                            tool_call.function.name.clone(),
                            tool_call.function.arguments,
                        ));
                        did_call_tool = true;
                        current_tool_call = Some(tool_call.function.name);
                    }
                }
            }

            if did_call_tool {
                let tool_call_message = Message::Assistant {
                    content: OneOrMany::many(tool_calls.clone())
                        .expect("Failed to create tool call message"),
                };
                self.history.push(tool_call_message);
                for (id, result) in &tool_results {
                    self.history.push(Message::User {
                        content: OneOrMany::one(UserContent::tool_result(
                            id,
                            OneOrMany::one(ToolResultContent::text(result)),
                        )),
                    });
                }
                current_prompt = self.history.pop().unwrap();
                did_call_tool = false;
            } else {
                return Ok(());
            }
        }
    }
}
