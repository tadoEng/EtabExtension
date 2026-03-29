use anyhow::Result;

use ext_api::stash::{self, StashPopConflict, StashPopConflictResolution, StashPopOptions};

use crate::args::{StashArgs, StashSubcommand};
use crate::output::OutputChannel;

use super::{ctx_from, prompt_stash_overwrite, prompt_stash_pop_conflict};

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: StashArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;

    match args.command {
        Some(StashSubcommand::List) => {
            let result = stash::stash_list(&ctx).await?;
            if out.is_human() {
                for entry in &result.stashes {
                    println!(
                        "{}  basedOn={}  {}",
                        entry.branch,
                        entry.based_on.as_deref().unwrap_or("-"),
                        entry.stashed_at
                    );
                    if let Some(description) = entry.description.as_deref() {
                        println!("  {description}");
                    }
                }
            }
            if out.is_shell() {
                for entry in &result.stashes {
                    println!("{}", entry.branch);
                }
            }
            out.json_value(&result)?;
        }
        Some(StashSubcommand::Pop) => {
            let result = match stash::stash_pop(&ctx, StashPopOptions::default()).await {
                Ok(result) => result,
                Err(err) if out.is_human() => {
                    if let Some(conflict) = err.downcast_ref::<StashPopConflict>() {
                        if !prompt_stash_pop_conflict(conflict)? {
                            return Ok(());
                        }
                        stash::stash_pop(
                            &ctx,
                            StashPopOptions {
                                conflict_resolution: Some(StashPopConflictResolution::Overwrite),
                            },
                        )
                        .await?
                    } else {
                        return Err(err);
                    }
                }
                Err(err) => return Err(err),
            };

            if out.is_human() {
                println!("✓ Restored stash for {}", result.branch);
                if let Some(version) = result.restored_based_on.as_deref() {
                    println!("  Based on: {version}");
                }
            }
            out.shell_line(result.branch.clone());
            out.json_value(&result)?;
        }
        Some(StashSubcommand::Drop(drop_args)) => {
            let result = stash::stash_drop(&ctx, drop_args.force).await?;
            out.human_line(format!("✓ Dropped stash for {}", result.branch));
            out.shell_line(result.branch.clone());
            out.json_value(&result)?;
        }
        None => {
            let result = match stash::stash_save(&ctx, args.message.as_deref(), false).await {
                Ok(result) => result,
                Err(err)
                    if out.is_human()
                        && err.downcast_ref::<ext_core::stash::StashExists>().is_some() =>
                {
                    if !prompt_stash_overwrite()? {
                        return Ok(());
                    }
                    stash::stash_save(&ctx, args.message.as_deref(), true).await?
                }
                Err(err) => return Err(err),
            };

            if out.is_human() {
                println!("✓ Saved stash for {}", result.branch);
                println!("  Path: {}", result.stash_path.display());
            }
            out.shell_line(result.branch.clone());
            out.json_value(&result)?;
        }
    }

    Ok(())
}
