use anyhow::Result;
use ext_api::run_calc;

use crate::args::CalcArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: CalcArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = run_calc(&ctx, &args.version)?;

    if out.is_human() {
        println!(
            "Calc artifact captured for {}/{}",
            result.branch, result.version_id
        );
        println!("  Results: {}", result.results_dir.display());
        println!("  Output : {}", result.calc_output_path.display());
    }

    out.shell_line(result.calc_output_path.display().to_string());
    out.json_value(&result)?;
    Ok(())
}
