use anyhow::Result;

use ext_api::branch;

use crate::args::BranchArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: BranchArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;

    if let Some(name) = args.delete.as_deref() {
        let result = branch::delete_branch(&ctx, name, args.force).await?;
        out.human_line(format!("✓ Deleted branch {}", result.name));
        out.shell_line(result.name.clone());
        out.json_value(&result)?;
        return Ok(());
    }

    if let Some(name) = args.name.as_deref() {
        let result = branch::create_branch(&ctx, name, args.from.as_deref()).await?;
        if out.is_human() {
            println!("✓ Created branch {}", result.name);
            println!("  From: {}", result.created_from);
            println!("  Working: {}", result.working_model_path.display());
        }
        out.shell_line(result.name.clone());
        out.json_value(&result)?;
        return Ok(());
    }

    let result = branch::list_branches(&ctx).await?;
    if out.is_human() {
        for branch in &result.branches {
            let active = if branch.is_active { "*" } else { " " };
            let latest = branch.latest_version.as_deref().unwrap_or("-");
            println!(
                "{active} {}  versions={}  latest={latest}",
                branch.name, branch.version_count
            );
        }
    }
    if out.is_shell() {
        for branch in &result.branches {
            println!("{}", branch.name);
        }
    }
    out.json_value(&result)?;
    Ok(())
}
