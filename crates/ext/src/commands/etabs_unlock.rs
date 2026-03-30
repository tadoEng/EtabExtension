use anyhow::Result;

use ext_api::etabs_unlock;

use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = etabs_unlock(&ctx).await?;

    if out.is_human() {
        println!("✓ Model unlocked");
        println!("  File: {}", result.file.display());
        println!("  Reopened: {}", result.reopened_for_unlock);
        println!("  Working Status: {}", result.arrival_status);
    }

    out.shell_line(result.arrival_status.to_string());
    out.json_value(&result)?;
    Ok(())
}
