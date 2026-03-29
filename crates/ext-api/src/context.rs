// ext-api::context — AppContext: the root dependency passed into every API fn
//
// Construction order:
//   1. Locate project root (walk up from cwd looking for .etabs-ext/)
//   2. Load Config (config.toml merged with config.local.toml)
//   3. Resolve sidecar path (config → ETABS_SIDECAR_PATH env → PATH)
//   4. AppContext is now ready to pass into any ext-api function
//
// SIDECAR PATH RESOLUTION LIVES HERE.
// This is the correct place because:
//   - Resolution needs Config (from ext-db)
//   - ext-db depends on ext-core
//   - ext-core owns SidecarClient (takes a plain PathBuf)
//   - Putting resolution in ext-core would create a circular dep
// Solution: ext-api resolves the path, constructs SidecarClient(path),
// and passes it down. ext-core::SidecarClient stays dep-free.

use anyhow::{Context, Result, bail};
use ext_core::sidecar::SidecarClient;
use ext_core::{branch, fs, version};
use ext_db::{StateFile, config::Config};
use std::path::{Path, PathBuf};

use crate::path_utils::normalize_path;

pub struct AppContext {
    /// Absolute path to the project root (parent of .etabs-ext/)
    pub project_root: PathBuf,

    /// Fully resolved config — config.local.toml merged over config.toml
    pub config: Config,

    /// Sidecar client, present only when the binary was located at context
    /// construction time via:
    ///   config.toml project.sidecar-path → ETABS_SIDECAR_PATH env → PATH
    ///
    /// RULE: never access this field directly. Always call `require_sidecar()`
    /// which returns a clear error when the sidecar is not configured.
    ///
    /// The ONE permitted exception is `ext status --verbose`, which degrades
    /// gracefully by reporting a warning instead of failing. That exception must
    /// be explicitly documented at the call site with a comment explaining why.
    pub(crate) sidecar: Option<SidecarClient>,
}

impl AppContext {
    /// Build a context from a project root directory.
    pub fn new(project_root: &Path) -> Result<Self> {
        let project_root = normalize_path(
            &project_root
                .canonicalize()
                .with_context(|| format!("Project root not found: {}", project_root.display()))?,
        );

        if !Config::config_dir(&project_root).is_dir() {
            bail!(
                "Not an ext repository: {}\n  Run: ext init",
                project_root.display()
            );
        }

        let config = Config::load(&project_root)
            .with_context(|| format!("Failed to load config from {}", project_root.display()))?;
        let sidecar = config
            .resolve_sidecar_path(&project_root)
            .map(SidecarClient::new);

        let ext_dir = Config::config_dir(&project_root);
        if let Ok(entries) = std::fs::read_dir(&ext_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !branch::exists(name, &ext_dir) {
                    continue;
                }
                let _ = version::cleanup_partial_snapshots(&path);
                let _ = fs::cleanup_stale_tmp(&path.join("working"));
            }
        }

        Ok(Self {
            project_root,
            config,
            sidecar,
        })
    }

    /// Walk up from `start` to find the project root (directory containing .etabs-ext/).
    /// Returns Err if no .etabs-ext/ found (not an ext repository).
    pub fn locate(start: &Path) -> Result<PathBuf> {
        let mut current = if start.is_file() {
            start
                .parent()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow::anyhow!("Invalid start path: {}", start.display()))?
        } else {
            start.to_path_buf()
        };

        loop {
            if current.join(".etabs-ext").is_dir() {
                return Ok(current);
            }
            if !current.pop() {
                bail!(
                    "Not an ext repository: {}\n  Run: ext init",
                    start.display()
                );
            }
        }
    }

    /// Convenience: locate project root from cwd, then build AppContext.
    pub fn from_cwd() -> Result<Self> {
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let root = Self::locate(&cwd)?;
        Self::new(&root)
    }

    /// Absolute path to the ext metadata directory.
    pub fn ext_dir(&self) -> PathBuf {
        Config::config_dir(&self.project_root)
    }

    /// Load state.json fresh from disk.
    pub fn load_state(&self) -> Result<StateFile> {
        StateFile::load(&self.project_root)
    }

    /// Persist state.json atomically.
    pub fn save_state(&self, state: &StateFile) -> Result<()> {
        state.save(&self.project_root)
    }

    /// THE MANDATED CALL SITE for all sidecar access.
    ///
    /// Every ext-api function that needs to talk to ETABS must call this.
    /// It returns a clear, user-facing error when the sidecar is not configured,
    /// instead of silently doing nothing or panicking.
    ///
    /// ## Correct usage
    /// ```ignore
    /// pub async fn etabs_open(ctx: &AppContext, file: &Path) -> Result<OpenModelData> {
    ///     let sidecar = ctx.require_sidecar()?;
    ///     sidecar.open_model(file, false, false).await.map_err(Into::into)
    /// }
    /// ```
    ///
    /// ## The only permitted exception
    /// `ext status --verbose` may call `ctx.sidecar.as_ref()` directly and
    /// degrade gracefully with a warning instead of failing. That call site
    /// must carry a comment explaining the exception. All other callers must
    /// use `require_sidecar()`.
    pub fn require_sidecar(&self) -> Result<&SidecarClient> {
        self.sidecar.as_ref().with_context(|| {
            "etab-cli sidecar not found.\n  \
             Set project.sidecar-path in .etabs-ext/config.local.toml or config.toml\n  \
             or set ETABS_SIDECAR_PATH environment variable"
        })
    }

    /// For use in tests — create with a known project root without validation.
    #[cfg(test)]
    pub fn for_test(project_root: PathBuf, config: Config) -> Self {
        Self {
            sidecar: None,
            project_root,
            config,
        }
    }
}
