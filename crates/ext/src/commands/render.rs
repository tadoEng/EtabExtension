use anyhow::Result;
use ext_api::render_version;

use crate::args::RenderArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: RenderArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = render_version(&ctx, &args.version, args.output_root.as_deref())?;

    if out.is_human() {
        println!("Rendered {} chart asset(s) for {}/{}", result.assets.len(), result.branch, result.version_id);
        println!("  Output : {}", result.asset_dir.display());
        for asset in &result.assets {
            println!("  - {}", asset.path.display());
        }
    }

    out.shell_line(result.asset_dir.display().to_string());
    out.json_value(&result)?;
    Ok(())
}
