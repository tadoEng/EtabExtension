use anyhow::Result;
use ext_api::export_results_file;

use crate::args::EtabsExportResultsArgs;
use crate::output::OutputChannel;

use super::to_absolute;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: EtabsExportResultsArgs,
) -> Result<()> {
    let project_root = global_project_path
        .map(|path| to_absolute(path))
        .transpose()?;
    let file = to_absolute(&args.file)?;
    let output_dir = to_absolute(&args.output_dir)?;
    let result = export_results_file(
        project_root.as_deref(),
        &file,
        &output_dir,
        args.units.as_deref(),
    )
    .await?;

    if out.is_human() {
        println!("✓ ETABS results exported");
        println!("  File: {}", result.file_path.display());
        println!("  Output: {}", result.output_dir.display());
        println!(
            "  Rows: {} total  |  Tables: {} ok / {} failed",
            result.total_row_count, result.succeeded_count, result.failed_count
        );
    }

    out.shell_line(result.output_dir.display().to_string());
    out.json_value(&result)?;
    Ok(())
}
