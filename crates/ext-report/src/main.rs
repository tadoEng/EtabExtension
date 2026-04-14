use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ext_calc::CalcRunner;
use ext_calc::code_params::CodeParams;
use ext_calc::output::CalcOutput;
use ext_db::config::Config;
use ext_render::{BaseReactionGroup, RenderConfig, render_all_svg};
use ext_report::{ChartRef, ReportProjectMeta, build_report_document, render_pdf, write_pdf};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let options = CliOptions::parse(env::args_os().skip(1))?;

    match options.command {
        Command::Preview(preview) => preview_report(preview),
    }
}

fn preview_report(options: PreviewOptions) -> Result<()> {
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

    let version_id = resolve_version_id(&results_dir, options.version_id.as_deref());
    let branch = resolve_branch(&results_dir, options.branch.as_deref());
    let calc_output =
        load_or_build_calc_output(&input_path, &results_dir, &config, &version_id, &branch)?;
    let rendered = render_all_svg(&calc_output, &build_render_config(&config))?;
    let charts = rendered
        .assets
        .iter()
        .map(|asset| ChartRef {
            logical_name: asset.logical_name.clone(),
            caption: asset.caption.clone(),
        })
        .collect::<Vec<_>>();
    let document = build_report_document(
        &calc_output,
        &charts,
        build_project_meta(&config, &version_id, &branch),
    );
    let svg_map = rendered
        .assets
        .into_iter()
        .map(|asset| (asset.logical_name, asset.svg))
        .collect();
    let pdf = render_pdf(&document, svg_map)?;

    let output_path = options.out.unwrap_or_else(default_preview_path);
    write_pdf(&output_path, &pdf)?;

    println!("Config Root : {}", config_root.display());
    println!("Results Dir : {}", results_dir.display());
    println!("Version     : {version_id}");
    println!("Branch      : {branch}");
    println!("Output PDF  : {}", output_path.display());
    Ok(())
}

fn load_or_build_calc_output(
    input_path: &Path,
    results_dir: &Path,
    config: &Config,
    version_id: &str,
    branch: &str,
) -> Result<CalcOutput> {
    if input_path.is_file()
        && input_path.extension().and_then(|value| value.to_str()) == Some("json")
    {
        return load_calc_output(input_path);
    }

    let params = CodeParams::from_config(config)?;
    CalcRunner::run_all(results_dir, results_dir, &params, version_id, branch)
        .with_context(|| format!("Failed to run calculations for '{}'", results_dir.display()))
}

fn load_calc_output(path: &Path) -> Result<CalcOutput> {
    let text =
        fs::read_to_string(path).with_context(|| format!("Failed to read '{}'", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("Invalid JSON in '{}'", path.display()))
}

fn build_project_meta(config: &Config, version_id: &str, branch: &str) -> ReportProjectMeta {
    ReportProjectMeta {
        project_name: config
            .project
            .name
            .clone()
            .unwrap_or_else(|| "Report Preview".to_string()),
        project_number: version_id.to_string(),
        reference: format!("{branch}/{version_id}"),
        engineer: "Preview".to_string(),
        checker: "Preview".to_string(),
        date: "Preview".to_string(),
        subject: "Fixture report preview".to_string(),
        scale: "NTS".to_string(),
        revision: "0".to_string(),
        sheet_prefix: "SK".to_string(),
    }
}

fn build_render_config(config: &Config) -> RenderConfig {
    let configured_groups = config
        .calc
        .base_reactions
        .pie_groups
        .iter()
        .map(|group| BaseReactionGroup {
            label: group.label.clone(),
            load_cases: group.load_cases.clone(),
        })
        .collect::<Vec<_>>();

    RenderConfig {
        base_reaction_groups: if configured_groups.is_empty() {
            default_base_reaction_groups()
        } else {
            configured_groups
        },
        ..RenderConfig::default()
    }
}

fn default_base_reaction_groups() -> Vec<BaseReactionGroup> {
    ["Dead", "SDL", "Live_red", "Live_unred"]
        .into_iter()
        .map(|name| BaseReactionGroup {
            label: name.to_string(),
            load_cases: vec![name.to_string()],
        })
        .collect()
}

fn default_preview_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("preview")
        .join("report.pdf")
}

#[derive(Debug)]
struct CliOptions {
    command: Command,
}

#[derive(Debug)]
enum Command {
    Preview(PreviewOptions),
}

#[derive(Debug)]
struct PreviewOptions {
    input_path: PathBuf,
    config_root: Option<PathBuf>,
    results_dir: Option<PathBuf>,
    version_id: Option<String>,
    branch: Option<String>,
    out: Option<PathBuf>,
}

impl CliOptions {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self> {
        let mut iter = args.into_iter();
        let Some(command) = iter.next() else {
            print_usage();
            bail!("Missing command");
        };
        let command = command
            .into_string()
            .map_err(|_| anyhow::anyhow!("Unsupported non-UTF8 CLI argument"))?;

        let parsed = match command.as_str() {
            "preview" => Command::Preview(PreviewOptions::parse(iter)?),
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                print_usage();
                bail!("Unknown command: {other}");
            }
        };

        Ok(Self { command: parsed })
    }
}

impl PreviewOptions {
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
                "--config-root" => config_root = Some(next_path(&mut iter, "--config-root")?),
                "--results-dir" => results_dir = Some(next_path(&mut iter, "--results-dir")?),
                "--version-id" => version_id = Some(next_string(&mut iter, "--version-id")?),
                "--branch" => branch = Some(next_string(&mut iter, "--branch")?),
                "--out" => out = Some(next_path(&mut iter, "--out")?),
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
                "Missing input path\n\nUsage: cargo run -p ext-report -- preview <path> [--config-root PATH] [--results-dir PATH] [--version-id ID] [--branch NAME] [--out FILE]"
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
        None if input_path.is_file() => input_path
            .parent()
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("Invalid file input path '{}'", input_path.display()))?,
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
    } else if input_path.is_dir() && input_path.join(".etabs-ext").is_dir() {
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

fn resolve_version_id(results_dir: &Path, override_id: Option<&str>) -> String {
    override_id.map(ToOwned::to_owned).unwrap_or_else(|| {
        if results_dir.file_name().and_then(|value| value.to_str()) == Some("results") {
            results_dir
                .parent()
                .and_then(Path::file_name)
                .and_then(|value| value.to_str())
                .unwrap_or("preview")
                .to_string()
        } else {
            results_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("preview")
                .to_string()
        }
    })
}

fn resolve_branch(results_dir: &Path, override_branch: Option<&str>) -> String {
    override_branch.map(ToOwned::to_owned).unwrap_or_else(|| {
        if results_dir.file_name().and_then(|value| value.to_str()) == Some("results") {
            results_dir
                .parent()
                .and_then(Path::parent)
                .and_then(Path::file_name)
                .and_then(|value| value.to_str())
                .unwrap_or("preview")
                .to_string()
        } else {
            "preview".to_string()
        }
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p ext-report -- preview <path> [--config-root PATH] [--results-dir PATH] [--version-id ID] [--branch NAME] [--out FILE]"
    );
    println!();
    println!("Examples:");
    println!(
        "  cargo run -p ext-report -- preview crates/ext-calc/tests/fixtures/results_realistic"
    );
    println!(
        "  cargo run -p ext-report -- preview crates/ext-calc/tests/fixtures/results_realistic --out proofs/output/results_realistic_report.pdf"
    );
    println!(
        "  cargo run -p ext-report -- preview crates/ext-calc/tests/fixtures/results_realistic/calc_output.json --config-root crates/ext-calc/tests/fixtures/results_realistic"
    );
}
