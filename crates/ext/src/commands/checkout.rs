use anyhow::Result;

use ext_api::checkout::{self, CheckoutConflict, CheckoutConflictResolution, CheckoutOptions};

use crate::args::CheckoutArgs;
use crate::output::OutputChannel;

use super::{ctx_from, prompt_checkout_conflict};

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: CheckoutArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let initial_opts = CheckoutOptions {
        conflict_resolution: if args.force {
            Some(CheckoutConflictResolution::Discard)
        } else {
            None
        },
    };

    let result = match checkout::checkout_version(&ctx, &args.version, initial_opts).await {
        Ok(result) => result,
        Err(err) if out.is_human() => {
            if let Some(conflict) = err.downcast_ref::<CheckoutConflict>() {
                let Some(resolution) = prompt_checkout_conflict(conflict)? else {
                    return Ok(());
                };
                checkout::checkout_version(
                    &ctx,
                    &args.version,
                    CheckoutOptions {
                        conflict_resolution: Some(resolution),
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
        println!("✓ Checked out {}/{}", result.branch, result.version_id);
        println!("  Working: {}", result.working_model_path.display());
    }
    out.shell_line(result.version_id.clone());
    out.json_value(&result)?;
    Ok(())
}
