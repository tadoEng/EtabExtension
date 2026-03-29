use anyhow::Result;

use ext_api::log as versions;

use crate::args::LogArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: LogArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = versions::list_versions(&ctx, args.branch.as_deref(), args.all).await?;

    if out.is_human() {
        println!("Branch: {}", result.branch);
        for commit in &result.commits {
            let version = commit.version_id.as_deref().unwrap_or("-");
            println!(
                "{}  {}  {}  {}",
                commit.hash, version, commit.author, commit.message
            );
        }
    }
    if out.is_shell() {
        for commit in &result.commits {
            println!(
                "{}",
                commit.version_id.as_deref().unwrap_or(commit.hash.as_str())
            );
        }
    }
    out.json_value(&result)?;
    Ok(())
}
