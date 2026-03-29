use anyhow::{Result, bail};

use ext_api::init::{self, InitRequest};

use crate::args::InitArgs;
use crate::output::OutputChannel;

use super::{prompt_onedrive_continue, to_absolute};

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
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
