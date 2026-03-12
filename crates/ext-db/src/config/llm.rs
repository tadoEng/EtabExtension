// ext-db::config::llm — [llm] section
//
// SECURITY: ALL llm fields belong in config.local.toml ONLY.
// config.toml is git-tracked and pushed to OneDrive.
// ext config set ai.* routing must enforce this.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LlmConfig {
    /// "claude", "ollama", "openai", "lmstudio", "azure"
    /// Default when unset: "ollama" (local-first policy from agents.md)
    pub provider: Option<String>,

    /// API key — NEVER in config.toml
    pub api_key: Option<String>,

    /// Model identifier, e.g. "claude-sonnet-4-20250514", "qwen2.5-coder:14b"
    pub model: Option<String>,

    /// Base URL for Ollama / LM Studio / Azure endpoints
    pub base_url: Option<String>,

    /// Skip confirmation gate for write tools (agent auto-confirm).
    /// Only for trusted automation — never default to true.
    pub auto_confirm: Option<bool>,
}

impl LlmConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            provider: other.provider.or(self.provider),
            api_key: other.api_key.or(self.api_key),
            model: other.model.or(self.model),
            base_url: other.base_url.or(self.base_url),
            auto_confirm: other.auto_confirm.or(self.auto_confirm),
        }
    }

    /// Effective provider, defaulting to "ollama" per local-first policy.
    pub fn provider_or_default(&self) -> &str {
        self.provider.as_deref().unwrap_or("ollama")
    }

    pub fn auto_confirm_or_default(&self) -> bool {
        self.auto_confirm.unwrap_or(false)
    }
}
