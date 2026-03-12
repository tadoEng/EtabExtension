use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Shell,
    Json,
}

pub struct OutputChannel {
    format: OutputFormat,
}

impl OutputChannel {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn is_human(&self) -> bool {
        matches!(self.format, OutputFormat::Human)
    }

    pub fn is_shell(&self) -> bool {
        matches!(self.format, OutputFormat::Shell)
    }

    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }

    pub fn human_line(&self, text: impl AsRef<str>) {
        if self.is_human() {
            println!("{}", text.as_ref());
        }
    }

    pub fn shell_line(&self, text: impl AsRef<str>) {
        if self.is_shell() {
            println!("{}", text.as_ref());
        }
    }

    pub fn json_value<T: serde::Serialize>(&self, value: &T) -> Result<()> {
        if self.is_json() {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
        Ok(())
    }
}
