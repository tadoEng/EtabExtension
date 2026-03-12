use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use ext_api::AppContext;
use ext_api::init::{self, InitRequest};
use ext_api::status::{self, StatusOptions};

mod output;
use output::{OutputChannel, OutputFormat};

#[derive(Debug, Parser)]
#[command(name = "ext")]
#[command(about = "ETABS extension CLI")]
struct Cli {
    #[arg(long, global = true, conflicts_with = "shell")]
    json: bool,

    #[arg(long, global = true)]
    shell: bool,

    #[arg(long, global = true)]
    project_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init(InitArgs),
    Status(StatusArgs),
}

#[derive(Debug, Args)]
struct InitArgs {
    name: String,

    #[arg(long)]
    edb: PathBuf,

    #[arg(long)]
    path: Option<PathBuf>,

    #[arg(long)]
    author: Option<String>,

    #[arg(long)]
    email: Option<String>,

    #[arg(long)]
    onedrive: Option<PathBuf>,

    #[arg(long)]
    reports: Option<PathBuf>,

    #[arg(long)]
    allow_onedrive: bool,
}

#[derive(Debug, Args)]
struct StatusArgs {
    #[arg(long)]
    verbose: bool,
}

fn to_absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn prompt_onedrive_continue() -> Result<bool> {
    println!("⚠ OneDrive path detected");
    println!("  ETABS model files can be corrupted by sync conflicts.");
    println!("  [c] Continue anyway  [x] Cancel");
    print!("> ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("c"))
}

async fn run_init(
    out: &OutputChannel,
    global_project_path: Option<&PathBuf>,
    args: InitArgs,
) -> Result<()> {
    let project_path = if let Some(path) = args.path.as_ref() {
        to_absolute(path)?
    } else if let Some(path) = global_project_path {
        to_absolute(path)?
    } else {
        std::env::current_dir()?
    };
    let edb_path = to_absolute(&args.edb)?;

    let mut allow_onedrive = args.allow_onedrive;
    let onedrive_detected =
        init::is_onedrive_path(&project_path) || init::is_onedrive_path(&edb_path);
    if onedrive_detected && !allow_onedrive {
        if out.is_human() {
            if !prompt_onedrive_continue()? {
                bail!("Init cancelled by user");
            }
            allow_onedrive = true;
        } else {
            bail!("OneDrive path detected. Re-run with --allow-onedrive");
        }
    }

    let result = init::init_project(InitRequest {
        name: args.name.clone(),
        edb_path,
        project_path,
        author: args.author,
        email: args.email,
        one_drive_dir: args.onedrive,
        reports_dir: args.reports,
        allow_onedrive,
    })
    .await?;

    out.human_line(format!("✓ Created project: {}", args.name));
    out.human_line("✓ Initialized git repository");
    out.human_line(format!(
        "✓ Copied model → {}",
        result.working_model_path.display()
    ));
    if result.onedrive_detected {
        out.human_line("⚠ Project path is inside OneDrive-synced folder");
    }
    out.human_line("→ Next: ext commit \"Initial model\"");

    out.shell_line(result.project_root.display().to_string());
    out.json_value(&result)?;

    Ok(())
}

async fn run_status(
    out: &OutputChannel,
    global_project_path: Option<&PathBuf>,
    args: StatusArgs,
) -> Result<()> {
    let ctx = if let Some(path) = global_project_path {
        AppContext::new(&to_absolute(path)?)?
    } else {
        AppContext::from_cwd()?
    };

    let report = status::project_status(
        &ctx,
        StatusOptions {
            verbose: args.verbose,
        },
    )
    .await?;

    if out.is_human() {
        let project_name = report
            .project_name
            .clone()
            .unwrap_or_else(|| "Unnamed Project".to_string());
        println!("Project: {project_name}");
        println!("Working File: {}", report.working_status);
        println!("  Path: {}", report.working_model_path.display());
        if let Some(version) = report.based_on_version.as_deref() {
            println!("  Based on: {version}");
        }
        if let Some(pid) = report.etabs_pid {
            println!("  ETABS PID: {pid}");
        }
        if args.verbose {
            println!("Project Root: {}", report.project_root.display());
            if let Some(sidecar) = report.sidecar_status.as_ref() {
                println!("Sidecar:");
                println!("  Running: {}", sidecar.is_running);
                if let Some(pid) = sidecar.pid {
                    println!("  PID: {pid}");
                }
                if let Some(path) = sidecar.open_file_path.as_deref() {
                    println!("  Open File: {path}");
                }
            }
            if let Some(warning) = report.sidecar_warning.as_deref() {
                println!("⚠ {warning}");
            }
        }
        if let Some(warning) = report.onedrive_warning.as_deref() {
            println!("⚠ {warning}");
        }
    }

    out.shell_line(report.working_status.to_string());
    out.json_value(&report)?;
    Ok(())
}

fn output_format(cli: &Cli) -> OutputFormat {
    if cli.json {
        OutputFormat::Json
    } else if cli.shell {
        OutputFormat::Shell
    } else {
        OutputFormat::Human
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let out = OutputChannel::new(output_format(&cli));

    match cli.command {
        Command::Init(args) => run_init(&out, cli.project_path.as_ref(), args).await,
        Command::Status(args) => run_status(&out, cli.project_path.as_ref(), args).await,
    }
    .with_context(|| "Command failed".to_string())
}
