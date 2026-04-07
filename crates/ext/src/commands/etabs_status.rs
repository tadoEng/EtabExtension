use anyhow::Result;

use ext_api::etabs_status;

use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = etabs_status(&ctx).await?;

    if out.is_human() {
        println!("ETABS Running: {}", result.is_running);
        println!("Working File: {}", result.working_file_status);
        if let Some(pid) = result.pid {
            println!("  PID: {pid}");
        }
        if let Some(path) = result.open_file_path.as_ref() {
            println!("  Open File: {}", path.display());
        }
        if let Some(version) = result.etabs_version.as_deref() {
            println!("  Version: {version}");
        }
        if let Some(locked) = result.is_locked {
            println!("  Locked: {locked}");
        }
        if let Some(analyzed) = result.is_analyzed {
            println!("  Analyzed: {analyzed}");
        }
    }

    out.shell_line(result.working_file_status.to_string());
    out.json_value(&result)?;
    Ok(())
}
