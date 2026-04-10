use anyhow::{Result, bail};
use ext_api::{run_calc, run_calc_for_results_dir};

use crate::args::CalcArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: CalcArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = match (&args.version, &args.results_dir) {
        (Some(version), None) => run_calc(&ctx, version)?,
        (None, Some(results_dir)) => run_calc_for_results_dir(&ctx, results_dir)?,
        _ => bail!("Specify either <version> or --results-dir <dir>"),
    };

    if out.is_human() {
        if let (Some(branch), Some(version_id)) = (&result.branch, &result.version_id) {
            println!("Calc artifact captured for {}/{}", branch, version_id);
        } else {
            println!("Calc artifact captured for direct results directory");
        }
        println!("  Results: {}", result.results_dir.display());
        println!("  Output : {}", result.calc_output_path.display());
    }

    out.shell_line(result.calc_output_path.display().to_string());
    out.json_value(&result)?;
    Ok(())
}
