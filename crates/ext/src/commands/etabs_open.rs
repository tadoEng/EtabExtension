use anyhow::Result;

use ext_api::etabs_open;

use crate::args::EtabsOpenArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: EtabsOpenArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = etabs_open(&ctx, args.version.as_deref()).await?;

    if let Some(warning) = result.warning.as_deref() {
        out.human_line(format!("⚠ {warning}"));
    }

    if out.is_human() {
        println!("✓ ETABS opened (PID: {})", result.pid);
        println!("  File: {}", result.opened_file.display());
        if result.is_snapshot {
            println!("  Snapshot: yes");
        }
    }

    out.shell_line(result.pid.to_string());
    out.json_value(&result)?;
    Ok(())
}
