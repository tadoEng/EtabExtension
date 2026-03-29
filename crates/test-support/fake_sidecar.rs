use ext_db::config::Config;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[allow(dead_code)]
pub enum FakeSidecarMode {
    Success,
    AnalysisFail,
}

pub fn write_fake_sidecar(temp: &TempDir, mode: FakeSidecarMode) -> PathBuf {
    let (name, mode_token) = match mode {
        FakeSidecarMode::Success => ("fake-sidecar-success.cmd", "success"),
        FakeSidecarMode::AnalysisFail => ("fake-sidecar-analysis-fail.cmd", "analysis-fail"),
    };

    let path = temp.path().join(name);
    let script = r#"@echo off
setlocal EnableDelayedExpansion
set "mode=__MODE__"
set "cmd=%~1"
shift
set "output_dir="
:parse
if "%~1"=="" goto parsed
if /I "%~1"=="--output-dir" (
  set "output_dir=%~2"
  shift
)
shift
goto parse
:parsed
if /I "%cmd%"=="run-analysis" goto run_analysis
if /I "%cmd%"=="extract-results" goto extract_results
echo {"success":false,"error":"unsupported command"}
exit /b 0

:run_analysis
if /I "!mode!"=="analysis-fail" (
  echo {"success":false,"error":"simulated analysis failure"}
  exit /b 0
)
echo {"success":true,"data":{"filePath":"fake.edb","casesRequested":null,"caseCount":2,"finishedCaseCount":2,"analysisTimeMs":1234,"units":{"force":"kip","length":"ft","temperature":"F","isUs":true,"isMetric":false,"rawForce":1,"rawLength":2,"rawTemperature":3}}}
exit /b 0

:extract_results
if not "!output_dir!"=="" (
  if not exist "!output_dir!" mkdir "!output_dir!" >nul 2>&1
  > "!output_dir!\base-reactions.parquet" echo parquet
)
echo {"success":true,"data":{"filePath":"fake.edb","outputDir":"results","tables":{"baseReactions":{"success":true,"outputFile":"results/base-reactions.parquet","rowCount":12,"discardedRowCount":0,"error":null,"extractionTimeMs":120}},"totalRowCount":12,"succeededCount":1,"failedCount":0,"units":{"force":"kip","length":"ft","temperature":"F","isUs":true,"isMetric":false,"rawForce":1,"rawLength":2,"rawTemperature":3},"extractionTimeMs":120}}
exit /b 0
"#
        .replace("__MODE__", mode_token);

    std::fs::write(&path, script).unwrap();
    path
}

pub fn configure_fake_sidecar(project_root: &Path, sidecar_path: &Path) {
    let mut config = Config::load(project_root).unwrap();
    config.project.sidecar_path = Some(sidecar_path.display().to_string());
    config.project.units = Some("kip-ft-F".to_string());
    Config::write_local(project_root, &config).unwrap();
}
