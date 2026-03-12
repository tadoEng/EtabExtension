use serde::{Deserialize, Serialize};

pub trait LlmClient {
    fn provider_name(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prompt {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Completion {
    pub content: String,
}

