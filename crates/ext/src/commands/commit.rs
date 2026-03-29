use anyhow::Result;

use ext_api::commit;

use crate::args::CommitArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: CommitArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = commit::commit_version(
        &ctx,
        &args.message,
        commit::CommitOptions {
            no_e2k: args.no_e2k,
            analyze: args.analyze,
        },
    )
    .await?;

    if let Some(warning) = result.warning.as_deref() {
        out.human_line(warning);
    }

    if out.is_human() {
        println!("✓ Version {} saved", result.version_id);
        println!("  Branch: {}  |  {}", result.branch, result.git_hash);
        if result.e2k_generated {
            println!("  E2K: {} KB", result.e2k_size_bytes.unwrap_or(0) / 1024);
        } else if args.no_e2k {
            println!("  E2K: skipped (--no-e2k)");
        }
        if args.analyze {
            println!(
                "  Analysis: {}",
                if result.analyzed {
                    "captured"
                } else {
                    "requested, but not finalized"
                }
            );
        }
    }
    out.shell_line(result.version_id.clone());
    out.json_value(&result)?;
    Ok(())
}
