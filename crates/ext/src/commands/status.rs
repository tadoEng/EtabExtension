use anyhow::Result;

use ext_api::status::{self, StatusOptions};

use crate::args::StatusArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: StatusArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
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
