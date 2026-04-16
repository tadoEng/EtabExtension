// ext-core::sidecar::client

use crate::sidecar::types::SidecarResponse;
use ext_error::{ExtError, ExtResult};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

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
    fn timeout_for(command_name: &str) -> Duration {
        match command_name {
            "get-status" => Duration::from_secs(15),
            "open-model" => Duration::from_secs(120),
            "close-model" => Duration::from_secs(30),
            "generate-e2k" => Duration::from_secs(300),
            "extract-results" => Duration::from_secs(300),
            "run-analysis" => Duration::from_secs(1800),
            _ => Duration::from_secs(60),
        }
    }

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
        let command_name = args.first().copied().unwrap_or("unknown");
        let timeout_window = Self::timeout_for(command_name);

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

        // Collect stdout — the sidecar writes exactly one JSON object, but it
        // may be pretty-printed across multiple lines. We must preserve the
        // full payload rather than keeping only the last non-empty line.
        let stdout = child.stdout.take().expect("stdout was piped");
        let mut stdout_reader = BufReader::new(stdout);
        let (json_payload, status) = match timeout(timeout_window, async {
            let mut json_payload = String::new();
            stdout_reader
                .read_to_string(&mut json_payload)
                .await
                .map_err(|e| ExtError::SidecarFailed {
                    code: -1,
                    stderr: e.to_string(),
                })?;

            let status = child.wait().await.map_err(|e| ExtError::SidecarFailed {
                code: -1,
                stderr: e.to_string(),
            })?;

            Ok::<_, ExtError>((json_payload, status))
        })
        .await
        {
            Ok(result) => result?,
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(ExtError::SidecarFailed {
                    code: -1,
                    stderr: format!(
                        "Sidecar command '{command_name}' timed out after {}s",
                        timeout_window.as_secs()
                    ),
                });
            }
        };

        let json_payload = json_payload.trim();

        if json_payload.is_empty() {
            return Err(ExtError::SidecarFailed {
                code: status.code().unwrap_or(-1),
                stderr: "No JSON output received from sidecar".to_string(),
            });
        }

        let response: SidecarResponse<T> = serde_json::from_str(json_payload)
            .map_err(|e| ExtError::SidecarParse(format!("{e}: raw={json_payload}")))?;

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

#[cfg(test)]
mod tests {
    use super::SidecarClient;
    use std::time::Duration;

    #[test]
    fn timeout_mapping_matches_manual_test_budget() {
        assert_eq!(
            SidecarClient::timeout_for("get-status"),
            Duration::from_secs(15)
        );
        assert_eq!(
            SidecarClient::timeout_for("open-model"),
            Duration::from_secs(120)
        );
        assert_eq!(
            SidecarClient::timeout_for("close-model"),
            Duration::from_secs(30)
        );
        assert_eq!(
            SidecarClient::timeout_for("generate-e2k"),
            Duration::from_secs(300)
        );
        assert_eq!(
            SidecarClient::timeout_for("extract-results"),
            Duration::from_secs(300)
        );
        assert_eq!(
            SidecarClient::timeout_for("run-analysis"),
            Duration::from_secs(1800)
        );
    }
}
