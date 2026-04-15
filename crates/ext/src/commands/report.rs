use anyhow::Result;
use ext_api::{ReportTheme, report_version};

use crate::args::{ReportArgs, ReportThemeArg};
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: ReportArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let theme = match args.theme {
        ReportThemeArg::Tabloid => ReportTheme::Tabloid,
        ReportThemeArg::A4 => ReportTheme::A4,
    };
    let result = report_version(
        &ctx,
        &args.version,
        args.output_root.as_deref(),
        &args.name,
        theme,
    )?;

    if out.is_human() {
        println!(
            "Report generated for {}/{}",
            result.branch, result.version_id
        );
        println!("  Theme  : {}", result.theme.as_str());
        println!("  PDF    : {}", result.pdf_path.display());
        println!("  Images : {}", result.logical_images.len());
    }

    out.shell_line(result.pdf_path.display().to_string());
    out.json_value(&result)?;
    Ok(())
}
