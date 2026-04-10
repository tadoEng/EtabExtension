use anyhow::Result;
use ext_api::analyze_file;

use crate::args::EtabsAnalyzeArgs;
use crate::output::OutputChannel;

use super::to_absolute;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: EtabsAnalyzeArgs,
) -> Result<()> {
    let project_root = global_project_path
        .map(|path| to_absolute(path))
        .transpose()?;
    let file = to_absolute(&args.file)?;
    let result = analyze_file(
        project_root.as_deref(),
        &file,
        args.units.as_deref(),
        args.cases.as_deref(),
    )
    .await?;

    if out.is_human() {
        println!("✓ ETABS analysis complete");
        println!("  File: {}", result.file_path.display());
        println!(
            "  Cases: {} finished / {} total",
            result.finished_case_count, result.case_count
        );
        println!("  Elapsed: {} ms", result.analysis_time_ms);
    }

    out.shell_line(result.file_path.display().to_string());
    out.json_value(&result)?;
    Ok(())
}
