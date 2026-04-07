// Dev review runner for ext-calc. The long-term public entry point remains ext-api/CLI.
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ext_calc::CalcRunner;
use ext_calc::code_params::CodeParams;
use ext_db::config::Config;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let options = CliOptions::parse(env::args_os().skip(1))?;

    let input_path = options.input_path.canonicalize().with_context(|| {
        format!(
            "Failed to resolve input path '{}'",
            options.input_path.display()
        )
    })?;
    let results_dir = resolve_results_dir(&input_path, options.results_dir.as_deref())?;
    let config_root =
        resolve_config_root(&input_path, &results_dir, options.config_root.as_deref())?;

    let config = Config::load(&config_root)
        .with_context(|| format!("Failed to load config from '{}'", config_root.display()))?;
    let params = CodeParams::from_config(&config)?;

    let version_id = options.version_id.unwrap_or_else(|| {
        results_dir
            .parent()
            .and_then(Path::file_name)
            .and_then(|value| value.to_str())
            .unwrap_or("review")
            .to_string()
    });
    let branch = options.branch.unwrap_or_else(|| "review".to_string());
    let output_path = options
        .out
        .unwrap_or_else(|| results_dir.join("calc_output.json"));

    let output = CalcRunner::run_all(&results_dir, &results_dir, &params, &version_id, &branch)
        .with_context(|| format!("Failed to run calculations for '{}'", results_dir.display()))?;

    let json = serde_json::to_string_pretty(&output)?;
    fs::write(&output_path, json)
        .with_context(|| format!("Failed to write '{}'", output_path.display()))?;

    println!("Config Root : {}", config_root.display());
    println!("Results Dir : {}", results_dir.display());
    println!("Version     : {}", version_id);
    println!("Branch      : {}", branch);
    println!("Output JSON : {}", output_path.display());
    println!(
        "Summary     : {} ({} pass / {} fail / {} checks)",
        output.summary.overall_status,
        output.summary.pass_count,
        output.summary.fail_count,
        output.summary.check_count
    );

    for line in &output.summary.lines {
        println!("- {} [{}] {}", line.key, line.status, line.message);
    }

    Ok(())
}

#[derive(Debug)]
struct CliOptions {
    input_path: PathBuf,
    config_root: Option<PathBuf>,
    results_dir: Option<PathBuf>,
    version_id: Option<String>,
    branch: Option<String>,
    out: Option<PathBuf>,
}

impl CliOptions {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self> {
        let mut input_path = None;
        let mut config_root = None;
        let mut results_dir = None;
        let mut version_id = None;
        let mut branch = None;
        let mut out = None;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            let value = arg
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Unsupported non-UTF8 CLI argument"))?;

            match value {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--config-root" => {
                    config_root = Some(next_path(&mut iter, "--config-root")?);
                }
                "--results-dir" => {
                    results_dir = Some(next_path(&mut iter, "--results-dir")?);
                }
                "--version-id" => {
                    version_id = Some(next_string(&mut iter, "--version-id")?);
                }
                "--branch" => {
                    branch = Some(next_string(&mut iter, "--branch")?);
                }
                "--out" => {
                    out = Some(next_path(&mut iter, "--out")?);
                }
                flag if flag.starts_with("--") => bail!("Unknown flag: {flag}"),
                path => {
                    if input_path.is_some() {
                        bail!("Only one input path may be provided");
                    }
                    input_path = Some(PathBuf::from(path));
                }
            }
        }

        let input_path = input_path.ok_or_else(|| {
            anyhow::anyhow!(
                "Missing input path\n\nUsage: cargo run -p ext-calc -- <path> [--config-root PATH] [--results-dir PATH] [--version-id ID] [--branch NAME] [--out FILE]"
            )
        })?;

        Ok(Self {
            input_path,
            config_root,
            results_dir,
            version_id,
            branch,
            out,
        })
    }
}

fn next_string(iter: &mut impl Iterator<Item = OsString>, flag: &str) -> Result<String> {
    iter.next()
        .ok_or_else(|| anyhow::anyhow!("Missing value for {flag}"))?
        .into_string()
        .map_err(|_| anyhow::anyhow!("Unsupported non-UTF8 value for {flag}"))
}

fn next_path(iter: &mut impl Iterator<Item = OsString>, flag: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_string(iter, flag)?))
}

fn resolve_results_dir(input_path: &Path, override_dir: Option<&Path>) -> Result<PathBuf> {
    let candidate = match override_dir {
        Some(path) => path.to_path_buf(),
        None => {
            let results_child = input_path.join("results");
            if results_child.is_dir() {
                results_child
            } else {
                input_path.to_path_buf()
            }
        }
    };

    if !candidate.is_dir() {
        bail!("Results directory '{}' does not exist", candidate.display());
    }

    Ok(candidate)
}

fn resolve_config_root(
    input_path: &Path,
    results_dir: &Path,
    override_root: Option<&Path>,
) -> Result<PathBuf> {
    let candidate = if let Some(path) = override_root {
        path.to_path_buf()
    } else if input_path.join(".etabs-ext").is_dir() {
        input_path.to_path_buf()
    } else if results_dir.join(".etabs-ext").is_dir() {
        results_dir.to_path_buf()
    } else if let Some(parent) = results_dir.parent() {
        if parent.join(".etabs-ext").is_dir() {
            parent.to_path_buf()
        } else {
            results_dir.to_path_buf()
        }
    } else {
        results_dir.to_path_buf()
    };

    if !candidate.join(".etabs-ext").is_dir() {
        bail!(
            "Config root '{}' does not contain .etabs-ext",
            candidate.display()
        );
    }

    Ok(candidate)
}

fn print_usage() {
    println!(
        "Usage: cargo run -p ext-calc -- <path> [--config-root PATH] [--results-dir PATH] [--version-id ID] [--branch NAME] [--out FILE]"
    );
    println!();
    println!("Examples:");
    println!("  cargo run -p ext-calc -- crates/ext-calc/tests/fixtures/results_realistic");
    println!("  cargo run -p ext-calc -- .etabs-ext/main/v1");
    println!(
        "  cargo run -p ext-calc -- .etabs-ext/main/v1 --out .etabs-ext/main/v1/results/calc_output.json"
    );
}
