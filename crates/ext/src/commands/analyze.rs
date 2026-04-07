use anyhow::Result;

use ext_api::{AnalyzeOptions, analyze_version};

use crate::args::AnalyzeArgs;
use crate::output::OutputChannel;

use super::ctx_from;

pub async fn execute(
    out: &OutputChannel,
    global_project_path: Option<&std::path::PathBuf>,
    args: AnalyzeArgs,
) -> Result<()> {
    let ctx = ctx_from(global_project_path)?;
    let result = analyze_version(
        &ctx,
        &args.version,
        AnalyzeOptions {
            cases: args.cases.clone(),
            force: args.force,
        },
    )
    .await?;

    if let Some(warning) = result.warning.as_deref() {
        out.human_line(warning);
    }

    if out.is_human() {
        println!(
            "✓ Analysis {} for {}/{}",
            if result.already_analyzed {
                "checked"
            } else {
                "captured"
            },
            result.branch,
            result.version_id
        );
        println!("  Results: {}", result.results_dir.display());
        println!("  Elapsed: {} ms", result.elapsed_ms);
    }

    out.shell_line(result.version_id.clone());
    out.json_value(&result)?;
    Ok(())
}
