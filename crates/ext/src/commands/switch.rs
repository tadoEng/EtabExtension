use anyhow::Result;

use ext_api::switch;

use crate::args::SwitchArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: SwitchArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = if args.create {
        switch::switch_and_create(&ctx, &args.name, args.from.as_deref()).await?
    } else {
        switch::switch_branch(&ctx, &args.name).await?
    };

    if out.is_human() {
        if let Some(warning) = result.departure_warning.as_deref() {
            println!("{warning}");
        }
        println!("✓ Switched to {}", result.branch);
        println!("  Status: {}", result.arrival_status);
        if let Some(warning) = result.arrival_warning.as_deref() {
            println!("{warning}");
        }
    }
    out.shell_line(result.branch.clone());
    out.json_value(&result)?;
    Ok(())
}
