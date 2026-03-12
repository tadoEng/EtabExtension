use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GitConfig {
    pub author: Option<String>,
    pub email: Option<String>,
}

impl GitConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            author: other.author.or(self.author),
            email: other.email.or(self.email),
        }
    }

    pub fn author_or_default(&self) -> &str {
        self.author.as_deref().unwrap_or("Unknown")
    }

    pub fn email_or_default(&self) -> &str {
        self.email.as_deref().unwrap_or("unknown@example.com")
    }
}
