use anyhow::Result;

use ext_api::diff;

use crate::args::DiffArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: DiffArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = diff::diff_versions(&ctx, &args.from, &args.to).await?;

    if out.is_human() {
        if let Some(warning) = result.no_e2k_warning.as_deref() {
            println!("{warning}");
        }
        print!("{}", result.diff_text);
    }
    if out.is_shell() {
        print!("{}", result.diff_text);
    }
    out.json_value(&result)?;
    Ok(())
}
