// ext-core::sidecar::client

use crate::sidecar::types::SidecarResponse;
use ext_error::{ExtError, ExtResult};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Spawns etab-cli.exe and captures the single JSON result from stdout.
///
/// The path is passed in already-resolved by ext-api::context::AppContext.
/// Resolution order (done in ext-api, not here):
///   1. config.toml → etabs.sidecarPath
///   2. ETABS_SIDECAR_PATH env var
///   3. PATH
///
/// stderr lines are streamed live to the terminal via tracing::info! so the
/// user sees ℹ ✓ ✗ ⚠ progress as it happens — never buffered.
pub struct SidecarClient {
    path: PathBuf,
}

impl SidecarClient {
    /// Create a client pointing at an already-resolved sidecar path.
    /// Call SidecarClient::locate() in ext-api to get the path first.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Run a sidecar command and parse the JSON response.
    ///
    /// `args` are passed directly to etab-cli.exe.
    /// Returns Err if the process fails to start, produces no output,
    /// produces unparseable output, or reports success=false in its JSON.
    pub async fn run<T>(&self, args: &[&str]) -> ExtResult<SidecarResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        if !self.path.exists() {
            return Err(ExtError::SidecarNotFound {
                path: self.path.display().to_string(),
            });
        }

        tracing::debug!("sidecar: {} {}", self.path.display(), args.join(" "));

        let mut child = Command::new(&self.path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExtError::SidecarFailed {
                code: -1,
                stderr: e.to_string(),
            })?;

        // Stream stderr live — C# writes ℹ ✓ ✗ ⚠ progress lines here.
        // We forward them via tracing so they surface in the terminal
        // regardless of whether the caller is the CLI or the agent.
        let stderr = child.stderr.take().expect("stderr was piped");
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                // tracing::info! maps to the terminal in human mode
                tracing::info!(target: "etab-cli", "{}", line);
            }
        });

        // Collect stdout — C# contract guarantees exactly one JSON object
        let stdout = child.stdout.take().expect("stdout was piped");
        let mut lines = BufReader::new(stdout).lines();
        let mut json_line = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                json_line = trimmed.to_string();
            }
        }

        let status = child.wait().await.map_err(|e| ExtError::SidecarFailed {
            code: -1,
            stderr: e.to_string(),
        })?;

        if json_line.is_empty() {
            return Err(ExtError::SidecarFailed {
                code: status.code().unwrap_or(-1),
                stderr: "No JSON output received from sidecar".to_string(),
            });
        }

        let response: SidecarResponse<T> = serde_json::from_str(&json_line)
            .map_err(|e| ExtError::SidecarParse(format!("{e}: raw={json_line}")))?;

        // Top-level success=false means a fatal error (ETABS didn't start,
        // file not found, etc.) — not a per-table partial failure.
        if !response.success {
            return Err(ExtError::SidecarError(
                response
                    .error
                    .unwrap_or_else(|| "Unknown sidecar error".to_string()),
            ));
        }

        Ok(response)
    }
}
