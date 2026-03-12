use ext_agent_llm::{Completion, LlmClient, Prompt};

pub struct Agent<C: LlmClient> {
    client: C,
}

impl<C: LlmClient> Agent<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub fn provider_name(&self) -> &'static str {
        self.client.provider_name()
    }

    pub fn build_prompt(&self, system: impl Into<String>, user: impl Into<String>) -> Prompt {
        Prompt {
            system: system.into(),
            user: user.into(),
        }
    }

    pub fn completion_from_text(&self, content: impl Into<String>) -> Completion {
        Completion {
            content: content.into(),
        }
    }
}

