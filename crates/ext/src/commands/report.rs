use anyhow::Result;
use ext_api::report_version;

use crate::args::ReportArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: ReportArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = report_version(&ctx, &args.version, args.output_root.as_deref(), &args.name)?;

    if out.is_human() {
        println!("Report generated for {}/{}", result.branch, result.version_id);
        println!("  PDF    : {}", result.pdf_path.display());
        println!("  Images : {}", result.logical_images.len());
    }

    out.shell_line(result.pdf_path.display().to_string());
    out.json_value(&result)?;
    Ok(())
}
