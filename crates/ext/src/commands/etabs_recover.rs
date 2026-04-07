use anyhow::Result;

use ext_api::{EtabsRecoverConflict, etabs_recover};

use crate::output::OutputChannel;

use super::{ctx_from, prompt_etabs_recover_conflict};

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = match etabs_recover(&ctx, None).await {
        Ok(result) => result,
        Err(err) => match err.downcast::<EtabsRecoverConflict>() {
            Ok(conflict) => {
                let Some(choice) = prompt_etabs_recover_conflict(&conflict)? else {
                    return Ok(());
                };
                etabs_recover(&ctx, Some(choice)).await?
            }
            Err(err) => return Err(err),
        },
    };

    if out.is_human() {
        println!("✓ Recovery completed");
        println!("  Choice: {:?}", result.choice);
        println!("  Working Status: {}", result.arrival_status);
        println!("  File: {}", result.working_file.display());
    }

    out.shell_line(result.arrival_status.to_string());
    out.json_value(&result)?;
    Ok(())
}
