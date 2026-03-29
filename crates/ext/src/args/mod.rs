// ext::args — clap argument structs.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

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
    Commit(CommitArgs),
    Log(LogArgs),
    Show(ShowArgs),
    Branch(BranchArgs),
    Switch(SwitchArgs),
    Checkout(CheckoutArgs),
    Stash(StashArgs),
    Diff(DiffArgs),
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
