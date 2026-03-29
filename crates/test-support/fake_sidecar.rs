use ext_db::config::Config;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[allow(dead_code)]
pub enum FakeSidecarMode {
    Success,
    AnalysisFail,
}

#[derive(Debug, Clone)]
pub struct FakeSidecarState {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub etabs_version: Option<String>,
    pub open_file_path: Option<PathBuf>,
    pub is_model_open: bool,
    pub is_locked: Option<bool>,
    pub is_analyzed: Option<bool>,
    pub unit_force: String,
    pub unit_length: String,
    pub unit_temperature: String,
    pub unit_is_us: bool,
    pub unit_is_metric: bool,
}

impl Default for FakeSidecarState {
    fn default() -> Self {
        Self {
            is_running: false,
            pid: Some(4242),
            etabs_version: Some("ETABS 22.0".to_string()),
            open_file_path: None,
            is_model_open: false,
            is_locked: None,
            is_analyzed: None,
            unit_force: "kip".to_string(),
            unit_length: "ft".to_string(),
            unit_temperature: "F".to_string(),
            unit_is_us: true,
            unit_is_metric: false,
        }
    }
}

pub fn sidecar_state_path(sidecar_path: &Path) -> PathBuf {
    sidecar_path.with_extension("state")
}

pub fn set_fake_sidecar_state(sidecar_path: &Path, state: &FakeSidecarState) {
    let path = sidecar_state_path(sidecar_path);
    let open_file = state
        .open_file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let etabs_version = state.etabs_version.clone().unwrap_or_default();
    let pid = state.pid.map(|pid| pid.to_string()).unwrap_or_default();
    let is_locked = state
        .is_locked
        .map(|flag| if flag { "1" } else { "0" })
        .unwrap_or_default();
    let is_analyzed = state
        .is_analyzed
        .map(|flag| if flag { "1" } else { "0" })
        .unwrap_or_default();

    let lines = [
        format!("is_running={}", if state.is_running { 1 } else { 0 }),
        format!("pid={pid}"),
        format!("etabs_version={etabs_version}"),
        format!("open_file_path={open_file}"),
        format!("is_model_open={}", if state.is_model_open { 1 } else { 0 }),
        format!("is_locked={is_locked}"),
        format!("is_analyzed={is_analyzed}"),
        format!("unit_force={}", state.unit_force),
        format!("unit_length={}", state.unit_length),
        format!("unit_temperature={}", state.unit_temperature),
        format!("unit_is_us={}", if state.unit_is_us { 1 } else { 0 }),
        format!(
            "unit_is_metric={}",
            if state.unit_is_metric { 1 } else { 0 }
        ),
    ];

    std::fs::write(path, lines.join("\n")).unwrap();
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
set "state_file=%~dpn0.state"
call :load_state
set "cmd=%~1"
shift
set "file="
set "output_dir="
set "save=0"
set "new_instance=0"
:parse
if "%~1"=="" goto parsed
if /I "%~1"=="--file" (
  set "file=%~2"
  shift
  shift
  goto parse
)
if /I "%~1"=="--output-dir" (
  set "output_dir=%~2"
  shift
  shift
  goto parse
)
if /I "%~1"=="--save" (
  set "save=1"
  shift
  goto parse
)
if /I "%~1"=="--no-save" (
  set "save=0"
  shift
  goto parse
)
if /I "%~1"=="--new-instance" (
  set "new_instance=1"
  shift
  goto parse
)
shift
goto parse
:parsed
if /I "%cmd%"=="get-status" goto get_status
if /I "%cmd%"=="open-model" goto open_model
if /I "%cmd%"=="close-model" goto close_model
if /I "%cmd%"=="unlock-model" goto unlock_model
if /I "%cmd%"=="run-analysis" goto run_analysis
if /I "%cmd%"=="extract-results" goto extract_results
echo {"success":false,"error":"unsupported command"}
exit /b 0

:default_state
set "is_running=0"
set "pid=4242"
set "etabs_version=ETABS 22.0"
set "open_file_path="
set "is_model_open=0"
set "is_locked="
set "is_analyzed="
set "unit_force=kip"
set "unit_length=ft"
set "unit_temperature=F"
set "unit_is_us=1"
set "unit_is_metric=0"
goto :eof

:load_state
call :default_state
if not exist "!state_file!" goto :eof
for /f "usebackq tokens=1* delims==" %%A in ("!state_file!") do (
  set "%%A=%%B"
)
goto :eof

:write_state
(
  echo is_running=!is_running!
  echo pid=!pid!
  echo etabs_version=!etabs_version!
  echo open_file_path=!open_file_path!
  echo is_model_open=!is_model_open!
  echo is_locked=!is_locked!
  echo is_analyzed=!is_analyzed!
  echo unit_force=!unit_force!
  echo unit_length=!unit_length!
  echo unit_temperature=!unit_temperature!
  echo unit_is_us=!unit_is_us!
  echo unit_is_metric=!unit_is_metric!
) > "!state_file!"
goto :eof

:nullable_number
set "%~2=null"
if not "%~1"=="" set "%~2=%~1"
goto :eof

:nullable_bool
set "%~2=null"
if /I "%~1"=="1" set "%~2=true"
if /I "%~1"=="0" set "%~2=false"
goto :eof

:nullable_string
set "%~2=null"
if "%~1"=="" goto :eof
set "__tmp=%~1"
set "__tmp=!__tmp:\=\\!"
set "%~2="!__tmp!""
goto :eof

:unit_system_json
set "%~1=null"
if /I "!is_running!"=="0" goto :eof
call :nullable_bool "!unit_is_us!" unit_is_us_json
call :nullable_bool "!unit_is_metric!" unit_is_metric_json
set "%~1={"force":"!unit_force!","length":"!unit_length!","temperature":"!unit_temperature!","isUs":!unit_is_us_json!,"isMetric":!unit_is_metric_json!}"
goto :eof

:get_status
call :nullable_bool "!is_running!" is_running_json
call :nullable_number "!pid!" pid_json
call :nullable_string "!etabs_version!" etabs_version_json
call :nullable_string "!open_file_path!" open_file_json
call :nullable_bool "!is_model_open!" is_model_open_json
call :nullable_bool "!is_locked!" is_locked_json
call :nullable_bool "!is_analyzed!" is_analyzed_json
call :unit_system_json unit_system_json
echo {"success":true,"data":{"isRunning":!is_running_json!,"pid":!pid_json!,"etabsVersion":!etabs_version_json!,"openFilePath":!open_file_json!,"isModelOpen":!is_model_open_json!,"isLocked":!is_locked_json!,"isAnalyzed":!is_analyzed_json!,"unitSystem":!unit_system_json!}}
exit /b 0

:open_model
call :nullable_string "!open_file_path!" previous_file_json
set "is_running=1"
if "!pid!"=="" set "pid=4242"
set "open_file_path=!file!"
set "is_model_open=1"
if "!is_locked!"=="" set "is_locked=0"
if "!is_analyzed!"=="" set "is_analyzed=0"
call :write_state
call :nullable_string "!file!" file_json
call :nullable_number "!pid!" pid_json
call :nullable_bool "!new_instance!" opened_in_new_instance_json
echo {"success":true,"data":{"filePath":!file_json!,"previousFilePath":!previous_file_json!,"pid":!pid_json!,"openedInNewInstance":!opened_in_new_instance_json!}}
exit /b 0

:close_model
call :nullable_string "!open_file_path!" closed_file_json
call :nullable_bool "!save!" was_saved_json
set "is_running=1"
set "is_model_open=0"
set "open_file_path="
set "is_locked="
set "is_analyzed="
call :write_state
echo {"success":true,"data":{"closedFilePath":!closed_file_json!,"wasSaved":!was_saved_json!}}
exit /b 0

:unlock_model
if /I "!is_model_open!"=="0" (
  echo {"success":false,"error":"file not open"}
  exit /b 0
)
if not "!file!"=="" if /I not "!file!"=="!open_file_path!" (
  echo {"success":false,"error":"requested file is not the active model"}
  exit /b 0
)
set "was_locked=!is_locked!"
if "!was_locked!"=="" set "was_locked=0"
set "is_locked=0"
call :write_state
call :nullable_string "!open_file_path!" file_json
call :nullable_bool "!was_locked!" was_locked_json
echo {"success":true,"data":{"filePath":!file_json!,"wasLocked":!was_locked_json!}}
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
    set_fake_sidecar_state(&path, &FakeSidecarState::default());
    path
}

pub fn configure_fake_sidecar(project_root: &Path, sidecar_path: &Path) {
    let mut config = Config::load(project_root).unwrap();
    config.project.sidecar_path = Some(sidecar_path.display().to_string());
    config.project.units = Some("kip-ft-F".to_string());
    Config::write_local(project_root, &config).unwrap();
}
