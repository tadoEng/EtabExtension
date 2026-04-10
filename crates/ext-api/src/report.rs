use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use ext_calc::{CalcRunner, code_params::CodeParams, output::CalcOutput};
use ext_core::vcs::current_branch;
use ext_render::{BaseReactionGroup, RenderConfig, render_all_svg, write_svg_assets};
use ext_report::{ChartRef, ReportProjectMeta, build_report_document, render_pdf, write_pdf};
use serde::{Deserialize, Serialize};

use crate::context::AppContext;

#[derive(Debug, Clone)]
struct VersionPaths {
    branch: String,
    version_id: String,
    version_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalcArtifacts {
    pub version_id: Option<String>,
    pub branch: Option<String>,
    pub results_dir: PathBuf,
    pub calc_output_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderArtifact {
    pub logical_name: String,
    pub caption: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderArtifacts {
    pub version_id: String,
    pub branch: String,
    pub output_dir: PathBuf,
    pub asset_dir: PathBuf,
    pub assets: Vec<RenderArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArtifacts {
    pub version_id: String,
    pub branch: String,
    pub output_dir: PathBuf,
    pub pdf_path: PathBuf,
    pub logical_images: Vec<String>,
}

pub fn run_calc(ctx: &AppContext, version_ref: &str) -> Result<CalcArtifacts> {
    let version = resolve_version_ref(ctx, version_ref)?;
    let results_dir = version.version_dir.join("results");
    if !results_dir.is_dir() {
        bail!(
            "Results directory missing for '{}/{}'. Run: ext analyze {}",
            version.branch,
            version.version_id,
            version_ref
        );
    }

    let params = CodeParams::from_config(&ctx.config)?;
    let calc_output = CalcRunner::run_all(
        &version.version_dir,
        &results_dir,
        &params,
        &version.version_id,
        &version.branch,
    )
    .with_context(|| {
        format!(
            "Failed to run calculations for '{}/{}'",
            version.branch, version.version_id
        )
    })?;

    let calc_output_path = calc_output_path(&version);
    let json = serde_json::to_string_pretty(&calc_output)?;
    std::fs::write(&calc_output_path, json)
        .with_context(|| format!("Failed to write {}", calc_output_path.display()))?;

    Ok(CalcArtifacts {
        version_id: Some(version.version_id),
        branch: Some(version.branch),
        results_dir,
        calc_output_path,
    })
}

pub fn run_calc_for_results_dir(ctx: &AppContext, results_dir: &Path) -> Result<CalcArtifacts> {
    if !results_dir.is_dir() {
        bail!("Results directory not found: {}", results_dir.display());
    }

    let params = CodeParams::from_config(&ctx.config)?;
    let calc_output =
        CalcRunner::run_all(results_dir, results_dir, &params, "direct", "direct").with_context(
            || format!("Failed to run calculations for {}", results_dir.display()),
        )?;

    let calc_output_path = results_dir.join("calc_output.json");
    let json = serde_json::to_string_pretty(&calc_output)?;
    std::fs::write(&calc_output_path, json)
        .with_context(|| format!("Failed to write {}", calc_output_path.display()))?;

    Ok(CalcArtifacts {
        version_id: None,
        branch: None,
        results_dir: results_dir.to_path_buf(),
        calc_output_path,
    })
}

pub fn load_calc_output(ctx: &AppContext, version_ref: &str) -> Result<CalcOutput> {
    let version = resolve_version_ref(ctx, version_ref)?;
    let path = calc_output_path(&version);
    let json = std::fs::read_to_string(&path)
        .with_context(|| format!("calc_output.json not found at {}", path.display()))?;
    serde_json::from_str(&json).with_context(|| format!("Invalid JSON at {}", path.display()))
}

pub fn render_version(
    ctx: &AppContext,
    version_ref: &str,
    output_root: Option<&Path>,
) -> Result<RenderArtifacts> {
    let version = resolve_version_ref(ctx, version_ref)?;
    let calc_output = load_calc_output(ctx, version_ref)?;
    let rendered = render_all_svg(&calc_output, &build_render_config(ctx))?;
    let output_dir = report_output_dir(ctx, &version, output_root);
    let asset_dir = output_dir.join("assets");
    let written_paths = write_svg_assets(&rendered, &asset_dir)?;

    let assets = rendered
        .assets
        .iter()
        .zip(written_paths)
        .map(|(asset, path)| RenderArtifact {
            logical_name: asset.logical_name.clone(),
            caption: asset.caption.clone(),
            path,
        })
        .collect();

    Ok(RenderArtifacts {
        version_id: version.version_id,
        branch: version.branch,
        output_dir,
        asset_dir,
        assets,
    })
}

pub fn report_version(
    ctx: &AppContext,
    version_ref: &str,
    output_root: Option<&Path>,
    report_name: &str,
) -> Result<ReportArtifacts> {
    let version = resolve_version_ref(ctx, version_ref)?;
    let calc_output = load_calc_output(ctx, version_ref)?;
    let rendered = render_all_svg(&calc_output, &build_render_config(ctx))?;
    let charts = rendered
        .assets
        .iter()
        .map(|asset| ChartRef {
            logical_name: asset.logical_name.clone(),
            caption: asset.caption.clone(),
        })
        .collect::<Vec<_>>();
    let document = build_report_document(&calc_output, &charts, build_project_meta(ctx, &version));
    let svg_map = rendered
        .assets
        .into_iter()
        .map(|asset| (asset.logical_name, asset.svg))
        .collect();
    let pdf = render_pdf(&document, svg_map)?;

    let output_dir = report_output_dir(ctx, &version, output_root);
    let pdf_path = output_dir.join(format!("{report_name}.pdf"));
    write_pdf(&pdf_path, &pdf)?;

    Ok(ReportArtifacts {
        version_id: version.version_id,
        branch: version.branch,
        output_dir,
        pdf_path,
        logical_images: charts.into_iter().map(|chart| chart.logical_name).collect(),
    })
}

fn resolve_version_ref(ctx: &AppContext, version_ref: &str) -> Result<VersionPaths> {
    let ext_dir = ctx.ext_dir();
    let current = current_branch(&ext_dir)?;
    let (branch, version_id) = if let Some((branch, version_id)) = version_ref.split_once('/') {
        (branch.to_string(), version_id.to_string())
    } else {
        (current, version_ref.to_string())
    };

    let version_dir = ext_dir.join(&branch).join(&version_id);
    if !version_dir.is_dir() {
        bail!(
            "✗ Version '{branch}/{version_id}' not found\n  Run: ext log to see available versions"
        );
    }

    Ok(VersionPaths {
        branch,
        version_id,
        version_dir,
    })
}

fn calc_output_path(version: &VersionPaths) -> PathBuf {
    version.version_dir.join("results").join("calc_output.json")
}

fn report_output_dir(
    ctx: &AppContext,
    version: &VersionPaths,
    output_root: Option<&Path>,
) -> PathBuf {
    resolve_reports_root(ctx, output_root)
        .join(&version.branch)
        .join(&version.version_id)
}

fn resolve_reports_root(ctx: &AppContext, output_root: Option<&Path>) -> PathBuf {
    match output_root {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => ctx.project_root.join(path),
        None => ctx
            .config
            .paths
            .reports_dir
            .as_deref()
            .map(PathBuf::from)
            .map(|path| {
                if path.is_absolute() {
                    path
                } else {
                    ctx.project_root.join(path)
                }
            })
            .unwrap_or_else(|| ctx.project_root.join("reports")),
    }
}

fn build_project_meta(ctx: &AppContext, version: &VersionPaths) -> ReportProjectMeta {
    let project_name = ctx
        .config
        .project
        .name
        .clone()
        .or_else(|| {
            ctx.project_root
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "ETABS Project".to_string());

    ReportProjectMeta {
        project_name,
        project_number: version.version_id.clone(),
        reference: format!("{}/{}", version.branch, version.version_id),
        engineer: ctx
            .config
            .git
            .author
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        checker: "Pending".to_string(),
        date: Utc::now().format("%Y-%m-%d").to_string(),
        subject: "Structural check report".to_string(),
        scale: "NTS".to_string(),
        revision: "0".to_string(),
        sheet_prefix: "SK".to_string(),
    }
}

fn build_render_config(ctx: &AppContext) -> RenderConfig {
    let configured_groups = ctx
        .config
        .calc
        .base_shear
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

#[cfg(test)]
mod tests {
    use super::{load_calc_output, render_version, report_version, run_calc, run_calc_for_results_dir};
    use crate::AppContext;
    use crate::commit::{CommitOptions, commit_version};
    use crate::init::{InitRequest, init_project};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn fixture_results_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic")
    }

    fn copy_fixture_config(project_root: &Path) {
        let fixture_root = fixture_results_dir();
        let fixture_config_dir = fixture_root.join(".etabs-ext");
        let project_config_dir = project_root.join(".etabs-ext");

        std::fs::copy(
            fixture_config_dir.join("config.toml"),
            project_config_dir.join("config.toml"),
        )
        .unwrap();
        std::fs::copy(
            fixture_config_dir.join("config.local.toml"),
            project_config_dir.join("config.local.toml"),
        )
        .unwrap();
    }

    async fn init_version_fixture(temp: &TempDir) -> PathBuf {
        let project_root = temp.path().join("project");
        let edb = temp.path().join("source").join("model.edb");
        std::fs::create_dir_all(edb.parent().unwrap()).unwrap();
        std::fs::write(&edb, b"dummy-edb-content").unwrap();

        init_project(InitRequest {
            name: "Fixture".to_string(),
            edb_path: edb,
            project_path: project_root.clone(),
            author: Some("Tester".to_string()),
            email: Some("tester@example.com".to_string()),
            one_drive_dir: None,
            reports_dir: None,
            allow_onedrive: true,
        })
        .await
        .unwrap();

        let ctx = AppContext::new(&project_root).unwrap();
        commit_version(
            &ctx,
            "Initial",
            CommitOptions {
                no_e2k: true,
                analyze: false,
            },
        )
        .await
        .unwrap();

        copy_fixture_config(&project_root);

        project_root
    }

    fn copy_fixture_results(project_root: &Path, include_calc_output: bool) {
        let source = fixture_results_dir();
        let target = project_root
            .join(".etabs-ext")
            .join("main")
            .join("v1")
            .join("results");
        std::fs::create_dir_all(&target).unwrap();

        for entry in std::fs::read_dir(&source).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().unwrap().to_string_lossy();
            if !include_calc_output && file_name == "calc_output.json" {
                continue;
            }

            std::fs::copy(&path, target.join(path.file_name().unwrap())).unwrap();
        }
    }

    #[tokio::test]
    async fn run_calc_writes_calc_output_json() {
        let temp = TempDir::new().unwrap();
        let project_root = init_version_fixture(&temp).await;
        copy_fixture_results(&project_root, false);
        let ctx = AppContext::new(&project_root).unwrap();

        let result = run_calc(&ctx, "v1").unwrap();
        assert!(result.calc_output_path.exists());
    }

    #[tokio::test]
    async fn run_calc_for_results_dir_writes_calc_output_json() {
        let temp = TempDir::new().unwrap();
        let project_root = init_version_fixture(&temp).await;
        copy_fixture_results(&project_root, false);
        let ctx = AppContext::new(&project_root).unwrap();
        let results_dir = project_root
            .join(".etabs-ext")
            .join("main")
            .join("v1")
            .join("results");

        let result = run_calc_for_results_dir(&ctx, &results_dir).unwrap();

        assert_eq!(result.version_id, None);
        assert_eq!(result.branch, None);
        assert!(result.calc_output_path.exists());
    }

    #[tokio::test]
    async fn load_calc_output_errors_when_missing() {
        let temp = TempDir::new().unwrap();
        let project_root = init_version_fixture(&temp).await;
        let ctx = AppContext::new(&project_root).unwrap();

        let err = load_calc_output(&ctx, "v1").unwrap_err();
        assert!(err.to_string().contains("calc_output.json"));
    }

    #[tokio::test]
    async fn render_version_writes_svg_assets() {
        let temp = TempDir::new().unwrap();
        let project_root = init_version_fixture(&temp).await;
        copy_fixture_results(&project_root, false);
        let ctx = AppContext::new(&project_root).unwrap();
        run_calc(&ctx, "v1").unwrap();

        let render = render_version(&ctx, "v1", None).unwrap();
        assert!(!render.assets.is_empty());
        assert!(render.assets.iter().all(|asset| asset.path.exists()));
    }

    #[tokio::test]
    async fn report_version_writes_pdf_without_prior_render_command() {
        let temp = TempDir::new().unwrap();
        let project_root = init_version_fixture(&temp).await;
        copy_fixture_results(&project_root, false);
        let ctx = AppContext::new(&project_root).unwrap();
        run_calc(&ctx, "v1").unwrap();

        let report = report_version(&ctx, "v1", None, "report").unwrap();
        assert!(report.pdf_path.exists());
    }
}
