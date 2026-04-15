// ext::args — clap argument structs.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "ext")]
#[command(about = "ETABS extension CLI")]
pub struct Cli {
    #[arg(long, global = true, conflicts_with = "shell")]
    pub json: bool,

    #[arg(long, global = true)]
    pub shell: bool,

    #[arg(long, global = true)]
    pub project_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init(InitArgs),
    Status(StatusArgs),
    Config(ConfigArgs),
    Commit(CommitArgs),
    Analyze(AnalyzeArgs),
    Calc(CalcArgs),
    Render(RenderArgs),
    Report(ReportArgs),
    Log(LogArgs),
    Show(ShowArgs),
    Branch(BranchArgs),
    Switch(SwitchArgs),
    Checkout(CheckoutArgs),
    Stash(StashArgs),
    Diff(DiffArgs),
    Etabs(EtabsArgs),
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    List,
    Get(ConfigGetArgs),
    Set(ConfigSetArgs),
}

#[derive(Debug, Args)]
pub struct ConfigGetArgs {
    pub key: String,
}

#[derive(Debug, Args)]
pub struct ConfigSetArgs {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    pub name: String,

    #[arg(long)]
    pub edb: PathBuf,

    #[arg(long)]
    pub path: Option<PathBuf>,

    #[arg(long)]
    pub author: Option<String>,

    #[arg(long)]
    pub email: Option<String>,

    #[arg(long)]
    pub onedrive: Option<PathBuf>,

    #[arg(long)]
    pub reports: Option<PathBuf>,

    #[arg(long)]
    pub allow_onedrive: bool,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Debug, Args)]
pub struct CommitArgs {
    pub message: String,

    #[arg(long)]
    pub analyze: bool,

    #[arg(long)]
    pub no_e2k: bool,
}

#[derive(Debug, Args)]
pub struct AnalyzeArgs {
    pub version: String,

    #[arg(long)]
    pub force: bool,

    #[arg(long, value_delimiter = ',')]
    pub cases: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct CalcArgs {
    pub version: Option<String>,

    #[arg(long, conflicts_with = "version")]
    pub results_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct RenderArgs {
    pub version: String,

    #[arg(long)]
    pub output_root: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ReportArgs {
    pub version: String,

    #[arg(long)]
    pub output_root: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = ReportThemeArg::Tabloid)]
    pub theme: ReportThemeArg,

    #[arg(long, default_value = "report")]
    pub name: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReportThemeArg {
    Tabloid,
    A4,
}

#[derive(Debug, Args)]
pub struct LogArgs {
    #[arg(long)]
    pub branch: Option<String>,

    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    pub version: String,
}

#[derive(Debug, Args)]
pub struct BranchArgs {
    pub name: Option<String>,

    #[arg(long)]
    pub from: Option<String>,

    #[arg(short = 'd', long = "delete")]
    pub delete: Option<String>,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct SwitchArgs {
    #[arg(short = 'c', long)]
    pub create: bool,

    pub name: String,

    #[arg(long)]
    pub from: Option<String>,
}

#[derive(Debug, Args)]
pub struct CheckoutArgs {
    pub version: String,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct StashArgs {
    #[arg(long)]
    pub message: Option<String>,

    #[command(subcommand)]
    pub command: Option<StashSubcommand>,
}

#[derive(Debug, Subcommand)]
pub enum StashSubcommand {
    List,
    Pop,
    Drop(StashDropArgs),
}

#[derive(Debug, Args)]
pub struct StashDropArgs {
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Args)]
pub struct EtabsArgs {
    #[command(subcommand)]
    pub command: EtabsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum EtabsSubcommand {
    Open(EtabsOpenArgs),
    Close(EtabsCloseArgs),
    Analyze(EtabsAnalyzeArgs),
    ExportResults(EtabsExportResultsArgs),
    Status,
    Unlock,
    Recover,
}

#[derive(Debug, Args)]
pub struct EtabsOpenArgs {
    pub version: Option<String>,

    /// Launch ETABS in a new instance instead of attaching to existing ETABS
    #[arg(long)]
    pub new_instance: bool,
}

#[derive(Debug, Args)]
pub struct EtabsCloseArgs {
    #[arg(long, conflicts_with = "no_save")]
    pub save: bool,

    #[arg(long, conflicts_with = "save")]
    pub no_save: bool,
}

#[derive(Debug, Args)]
pub struct EtabsAnalyzeArgs {
    #[arg(long)]
    pub file: PathBuf,

    #[arg(long)]
    pub units: Option<String>,

    #[arg(long, value_delimiter = ',')]
    pub cases: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct EtabsExportResultsArgs {
    #[arg(long)]
    pub file: PathBuf,

    #[arg(long)]
    pub output_dir: PathBuf,

    #[arg(long)]
    pub units: Option<String>,
}
