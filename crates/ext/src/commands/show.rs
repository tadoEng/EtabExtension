use anyhow::Result;

use ext_api::log as versions;

use crate::args::ShowArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: ShowArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = versions::show_version(&ctx, &args.version).await?;

    if out.is_human() {
        println!("Version: {}/{}", result.manifest.branch, result.manifest.id);
        println!("Message: {}", result.manifest.message);
        println!("Author: {}", result.manifest.author);
        println!("Timestamp: {}", result.manifest.timestamp);
        if let Some(parent) = result.manifest.parent.as_deref() {
            println!("Parent: {parent}");
        }
        println!("E2K Generated: {}", result.manifest.e2k_generated);
        println!(
            "Materials Extracted: {}",
            result.manifest.materials_extracted
        );
        if let Some(hash) = result.manifest.git_commit_hash.as_deref() {
            println!("Git Commit: {hash}");
        }
        if result.analysis.is_some() {
            println!("Analysis: present");
        }
    }
    out.shell_line(result.manifest.id.clone());
    out.json_value(&result)?;
    Ok(())
}
