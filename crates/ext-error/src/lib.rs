// ext-error — all ExtError variants
//
// Every crate in the workspace returns ExtError or wraps it via anyhow.
// Error messages follow the agents.md standard:
//   ✗ <what failed>
//     <why it failed>
//     Run: <command to fix it>

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtError {
    // ── Config / setup ─────────────────────────────────────────────────────
    #[error("Config not found at {path}")]
    ConfigNotFound { path: String },

    #[error("Config parse error in {path}: {detail}")]
    ConfigParse { path: String, detail: String },

    #[error("Invalid unit preset '{preset}'. Valid: {valid}")]
    InvalidUnitPreset { preset: String, valid: String },

    // ── State machine ──────────────────────────────────────────────────────
    #[error("Working file is in state {state}, which does not allow '{command}'\n  Run: {remedy}")]
    CommandGuard { state: String, command: String, remedy: String },

    #[error("State file corrupted: {0}")]
    StateCorrupted(String),

    #[error("No working file is tracked in this project\n  Run: ext init")]
    NoWorkingFile,

    // ── VCS ────────────────────────────────────────────────────────────────
    #[error("Not an ext repository: {path}\n  Run: ext init")]
    NotARepository { path: String },

    #[error("Version '{0}' not found")]
    VersionNotFound(String),

    #[error("Branch '{0}' already exists")]
    BranchExists(String),

    #[error("Branch '{0}' not found")]
    BranchNotFound(String),

    #[error("No versions yet — repository is empty\n  Run: ext commit \"initial\"")]
    EmptyRepository,

    #[error("Working file has uncommitted changes\n  Run: ext commit \"message\" or ext stash")]
    DirtyWorkingFile,

    #[error("Git error: {0}")]
    GitError(String),

    // ── Sidecar (C# etab-cli) ──────────────────────────────────────────────
    #[error("Sidecar binary not found at {path}\n  Check: etabs.sidecarPath in .etabs-ext/config.toml")]
    SidecarNotFound { path: String },

    #[error("Sidecar process failed (exit {code}): {stderr}")]
    SidecarFailed { code: i32, stderr: String },

    #[error("Sidecar response parse error: {0}")]
    SidecarParse(String),

    #[error("Sidecar reported failure: {0}")]
    SidecarError(String),

    #[error("ETABS is not running\n  Open ETABS and try again")]
    EtabsNotRunning,

    #[error("ETABS file is currently open\n  Close ETABS before continuing\n  Run: ext etabs close")]
    EtabsFileOpen { pid: u32 },

    #[error("ETABS file mismatch: expected {expected}, got {actual}")]
    EtabsFileMismatch { expected: String, actual: String },

    #[error("Working file state is unknown — ETABS may have crashed\n  Run: ext etabs recover")]
    WorkingFileOrphaned,

    // ── OneDrive ───────────────────────────────────────────────────────────
    #[error("Project is inside a OneDrive-synced folder\n  This can corrupt .edb files during sync")]
    OneDriveConflict,

    #[error("OneDrive folder not configured\n  Run: ext config set paths.oneDriveDir \"<path>\"")]
    OneDriveNotConfigured,

    // ── File I/O ───────────────────────────────────────────────────────────
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File already exists: {0}\n  Use --overwrite to replace")]
    FileExists(String),

    #[error("Atomic copy failed: {0}")]
    AtomicCopyFailed(String),

    #[error("Insufficient disk space\n  Need {required_mb} MB, have {available_mb} MB")]
    InsufficientDiskSpace { required_mb: u64, available_mb: u64 },

    // ── Database ───────────────────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(String),

    // ── Agent / LLM ────────────────────────────────────────────────────────
    #[error("LLM provider not configured\n  Run: ext config set ai.provider claude")]
    LlmNotConfigured,

    #[error("LLM request failed: {0}")]
    LlmRequest(String),

    #[error("Tool '{0}' not found in registry")]
    ToolNotFound(String),

    #[error("Tool '{0}' requires user confirmation")]
    ToolRequiresConfirmation(String),

    // ── Reports ────────────────────────────────────────────────────────────
    #[error("Report generation failed: {0}")]
    ReportFailed(String),

    #[error("Parquet read error at {path}: {detail}")]
    ParquetRead { path: String, detail: String },

    // ── Catch-all ──────────────────────────────────────────────────────────
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type ExtResult<T> = Result<T, ExtError>;
