use anyhow::Result;

use ext_api::{CloseMode, EtabsCloseConflict, etabs_close};

use crate::args::EtabsCloseArgs;
use crate::output::OutputChannel;

use super::{ctx_from, prompt_etabs_close_conflict};

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: EtabsCloseArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let mode = if args.save {
        CloseMode::Save
    } else if args.no_save {
        CloseMode::NoSave
    } else {
        CloseMode::Interactive
    };

    let result = match etabs_close(&ctx, mode).await {
        Ok(result) => result,
        Err(err) => match err.downcast::<EtabsCloseConflict>() {
            Ok(conflict) => {
                let Some(choice) = prompt_etabs_close_conflict(&conflict)? else {
                    return Ok(());
                };
                etabs_close(&ctx, choice).await?
            }
            Err(err) => return Err(err),
        },
    };

    if let Some(warning) = result.warning.as_deref() {
        out.human_line(format!("⚠ {warning}"));
    }

    if out.is_human() {
        println!("✓ ETABS model closed");
        println!("  Saved: {}", result.saved);
        println!("  Working Status: {}", result.arrival_status);
    }

    out.shell_line(result.arrival_status.to_string());
    out.json_value(&result)?;
    Ok(())
}
