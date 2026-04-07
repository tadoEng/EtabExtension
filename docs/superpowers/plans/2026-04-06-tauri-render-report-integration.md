# Tauri Render Report Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a consistent `ext-render` + `ext-report` + Tauri desktop integration that uses the persisted `calc_output.json` contract, tabloid report sizing, and explicit desktop commands for chart viewing and report generation through `ext-api`.

**Architecture:** `ext-calc` remains the source of engineering truth and persists `calc_output.json` as the canonical cross-frontend artifact. `ext-render` builds the display/report artifacts in memory, `ext-report` turns report-facing SVG maps into written Typst/PDF assets, `ext-api` owns workflow orchestration for both CLI and Tauri, and `ext-tauri` plus `apps/desktop` act as the desktop adapter and UI layer.

**Tech Stack:** Rust workspace crates, Tauri 2, React 19, TypeScript, Typst CLI, charming (`HtmlRenderer`/`ImageRenderer`)

---

### Task 1: Finalize `ext-render` crate contract

**Files:**
- Modify: `D:\Work\EtabExtension\crates\ext-render\Cargo.toml`
- Modify: `D:\Work\EtabExtension\crates\ext-render\src\lib.rs`
- Create: `D:\Work\EtabExtension\crates\ext-render\src\chart_build\mod.rs`
- Create: `D:\Work\EtabExtension\crates\ext-render\src\chart_build\drift.rs`
- Create: `D:\Work\EtabExtension\crates\ext-render\src\render_html\mod.rs`
- Create: `D:\Work\EtabExtension\crates\ext-render\src\render_svg\mod.rs`
- Test: `D:\Work\EtabExtension\crates\ext-render\tests\render_contract.rs`

- [ ] **Step 1: Write the failing render contract test**

```rust
use ext_calc::output::CalcOutput;
use ext_render::{render_all_html, RenderTheme};

#[test]
fn render_all_html_returns_named_chart_fragments() {
    let calc: CalcOutput = serde_json::from_str(include_str!(
        "../../ext-calc/tests/fixtures/results_realistic/calc_output.json"
    ))
    .unwrap();
    let charts = render_all_html(&calc, 960, 540, RenderTheme::Dark).unwrap();

    assert!(charts.contains_key("drift_wind"));
    assert!(charts["drift_wind"].contains("echarts"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p ext-render --test render_contract --target-dir .codex-target`
Expected: FAIL with unresolved items such as `render_all_html`, `RenderTheme`, or missing modules.

- [ ] **Step 3: Write the minimal crate contract**

```rust
#[derive(Debug, Clone, Copy)]
pub enum RenderTheme {
    Dark,
    Report,
}

pub fn render_all_html(
    calc: &CalcOutput,
    width: u32,
    height: u32,
    theme: RenderTheme,
) -> anyhow::Result<HashMap<String, String>>;

#[cfg(feature = "ssr")]
pub fn render_all_svg(
    calc: &CalcOutput,
) -> anyhow::Result<HashMap<String, String>>;
```

- [ ] **Step 4: Run targeted tests**

Run: `cargo test -p ext-render --test render_contract --target-dir .codex-target`
Expected: PASS, with chart keys matching the desktop contract and no filesystem writes in the HTML path.

- [ ] **Step 5: Commit**

```bash
git add crates/ext-render/Cargo.toml crates/ext-render/src/lib.rs crates/ext-render/src/chart_build crates/ext-render/src/render_html crates/ext-render/src/render_svg crates/ext-render/tests/render_contract.rs
git commit -m "feat: finalize ext-render contract"
```

### Task 2: Implement tabloid report composition from SVG maps

**Files:**
- Modify: `D:\Work\EtabExtension\crates\ext-report\Cargo.toml`
- Modify: `D:\Work\EtabExtension\crates\ext-report\src\lib.rs`
- Create: `D:\Work\EtabExtension\crates\ext-report\src\pdf\mod.rs`
- Create: `D:\Work\EtabExtension\crates\ext-report\src\pdf\renderer.rs`
- Create: `D:\Work\EtabExtension\crates\ext-report\src\pdf\template.rs`
- Test: `D:\Work\EtabExtension\crates\ext-report\tests\pdf_contract.rs`

- [ ] **Step 1: Write the failing report artifact test**

```rust
use std::collections::HashMap;

use ext_calc::output::CalcOutput;
use ext_report::render_pdf;
use tempfile::tempdir;

#[test]
fn render_pdf_writes_typst_and_assets_before_compile() {
    let calc: CalcOutput = serde_json::from_str(include_str!(
        "../../ext-calc/tests/fixtures/results_realistic/calc_output.json"
    ))
    .unwrap();
    let mut svg_map = HashMap::new();
    svg_map.insert("images/drift_wind.svg".to_string(), "<svg/>".to_string());

    let out = tempdir().unwrap();
    let paths = render_pdf(&calc, &svg_map, out.path(), "tower_a").unwrap();

    assert!(paths.assets_dir.exists());
    assert!(paths.typ.unwrap().exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p ext-report --test pdf_contract --target-dir .codex-target`
Expected: FAIL because `render_pdf` still has the old stubbed surface or does not write artifacts.

- [ ] **Step 3: Implement the report-facing API**

```rust
pub struct ReportPaths {
    pub pdf: PathBuf,
    pub typ: Option<PathBuf>,
    pub excel: Option<PathBuf>,
    pub assets_dir: PathBuf,
    pub assets: Vec<PathBuf>,
}

pub fn render_pdf(
    calc: &CalcOutput,
    svg_map: &HashMap<String, String>,
    output_dir: &Path,
    report_name: &str,
) -> anyhow::Result<ReportPaths>;
```

```typst
#set text(font: "Arial", size: 10pt)
#set page(width: 17in, height: 11in, margin: (x: 0.5in, y: 0.5in))

= #project_name

== Summary
#summary_table

== Drift Wind
#image("assets/drift_wind.svg", width: 15.5in)
```

- [ ] **Step 4: Run targeted tests**

Run: `cargo test -p ext-report --test pdf_contract --target-dir .codex-target`
Expected: PASS, with `report.typ` written using tabloid sizing and asset files created from the SVG map.

- [ ] **Step 5: Commit**

```bash
git add crates/ext-report/Cargo.toml crates/ext-report/src/lib.rs crates/ext-report/src/pdf crates/ext-report/tests/pdf_contract.rs
git commit -m "feat: add report svg-map pipeline"
```

### Task 3: Add `ext-api` render/report workflows

**Files:**
- Modify: `D:\Work\EtabExtension\crates\ext-api\src\lib.rs`
- Modify: `D:\Work\EtabExtension\crates\ext-api\src\report.rs`
- Create: `D:\Work\EtabExtension\crates\ext-api\src\render.rs`
- Modify: `D:\Work\EtabExtension\crates\ext-api\src\context.rs`
- Test: `D:\Work\EtabExtension\crates\ext-api\src\report.rs`

- [ ] **Step 1: Write the failing workflow-level contract test**

```rust
#[test]
fn generate_report_artifacts_returns_paths_from_app_context() {
    let ctx = test_app_context();
    let result = generate_report_artifacts(&ctx, "tower_a", ctx.project_root().join("out"));

    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo check -p ext-api --target-dir .codex-target`
Expected: FAIL because the render/report orchestration workflows do not exist yet.

- [ ] **Step 3: Implement the workflow surface**

```rust
pub fn get_rendered_charts(
    ctx: &AppContext,
    request: &ChartRequest,
) -> anyhow::Result<HashMap<String, String>>;

pub fn generate_report_artifacts(
    ctx: &AppContext,
    report_name: &str,
    output_dir: &Path,
) -> anyhow::Result<ReportPaths>;

pub fn load_calc_output(
    ctx: &AppContext,
) -> anyhow::Result<ext_calc::output::CalcOutput>;
```

- [ ] **Step 4: Run workflow compile checks**

Run: `cargo check -p ext-api --target-dir .codex-target`
Expected: PASS, with `ext-api` depending on `ext-render` and `ext-report`, loading the persisted calc artifact, and remaining the single orchestration layer.

- [ ] **Step 5: Commit**

```bash
git add crates/ext-api/src/lib.rs crates/ext-api/src/report.rs crates/ext-api/src/render.rs crates/ext-api/src/context.rs
git commit -m "feat: add ext-api render report workflows"
```

### Task 4: Add Tauri backend state and commands for charts and reports

**Files:**
- Modify: `D:\Work\EtabExtension\crates\ext-tauri\Cargo.toml`
- Modify: `D:\Work\EtabExtension\crates\ext-tauri\src\lib.rs`
- Modify: `D:\Work\EtabExtension\crates\ext-tauri\src\commands.rs`
- Create: `D:\Work\EtabExtension\crates\ext-tauri\src\state.rs`
- Modify: `D:\Work\EtabExtension\crates\ext-tauri\tauri.conf.json`
- Create: `D:\Work\EtabExtension\crates\ext-tauri\resources\echarts\echarts.min.js`
- Test: `D:\Work\EtabExtension\crates\ext-tauri\src\commands.rs`

- [ ] **Step 1: Write the failing backend command tests**

```rust
#[test]
fn render_all_charts_returns_error_without_calc_output() {
    let request = ChartRequest { width: 960, height: 540, theme: "dark".into() };

    let err = render_all_charts(None, &request).unwrap_err();
    assert!(err.contains("No calculation run yet"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo check -p ext-tauri --target-dir .codex-target`
Expected: FAIL because `AppState`, `ChartRequest`, and render/report command handlers do not exist yet.

- [ ] **Step 3: Implement app state and command DTOs**

```rust
#[derive(Default)]
pub struct AppState {
    pub calc_output: parking_lot::Mutex<Option<ext_calc::output::CalcOutput>>,
}

#[derive(serde::Deserialize)]
pub struct ChartRequest {
    pub width: u32,
    pub height: u32,
    pub theme: String,
}

#[derive(serde::Serialize)]
pub struct ReportArtifactDto {
    pub pdf_path: String,
    pub typ_path: Option<String>,
    pub asset_dir: String,
    pub asset_files: Vec<String>,
}
```

```rust
#[tauri::command]
pub fn get_rendered_charts(
    request: ChartRequest,
    state: tauri::State<'_, AppState>,
) -> Result<HashMap<String, String>, String>;

pub fn render_all_charts(
    calc: Option<&ext_calc::output::CalcOutput>,
    request: &ChartRequest,
) -> Result<HashMap<String, String>, String>;

#[tauri::command]
pub fn generate_report_artifacts(
    report_name: String,
    output_dir: String,
    state: tauri::State<'_, AppState>,
) -> Result<ReportArtifactDto, String>;
```

- [ ] **Step 4: Wire resources and compile**

Run: `cargo check -p ext-tauri --features report-ssr --target-dir .codex-target`
Expected: PASS, with `ext-tauri` forwarding the `report-ssr` feature, delegating workflows to `ext-api`, and `tauri.conf.json` listing `resources/echarts/echarts.min.js`.

- [ ] **Step 5: Commit**

```bash
git add crates/ext-tauri/Cargo.toml crates/ext-tauri/src/lib.rs crates/ext-tauri/src/commands.rs crates/ext-tauri/src/state.rs crates/ext-tauri/tauri.conf.json crates/ext-tauri/resources/echarts/echarts.min.js
git commit -m "feat: add tauri render report commands"
```

### Task 5: Replace desktop command typing and query plumbing

**Files:**
- Modify: `D:\Work\EtabExtension\apps\desktop\src\types\tauri-commands.ts`
- Create: `D:\Work\EtabExtension\apps\desktop\src\store\reportStore.ts`
- Create: `D:\Work\EtabExtension\apps\desktop\src\store\analysisStore.ts`
- Test: `D:\Work\EtabExtension\apps\desktop\src\types\tauri-commands.ts`

- [ ] **Step 1: Write the failing TypeScript contract types**

```ts
export type ChartRequest = {
    width: number;
    height: number;
    theme: "dark" | "report";
};

export type ReportArtifactDto = {
    pdfPath: string;
    typPath?: string | null;
    assetDir: string;
    assetFiles: string[];
};
```

- [ ] **Step 2: Run frontend typecheck to verify it fails**

Run: `pnpm --filter desktop build`
Expected: FAIL until the new desktop stores and command wrappers are added.

- [ ] **Step 3: Implement invoke wrappers and stores**

```ts
export async function getRenderedCharts(request: ChartRequest): Promise<Record<string, string>> {
    return invoke("get_rendered_charts", { request });
}

export async function generateReportArtifacts(input: {
    reportName: string;
    outputDir: string;
}): Promise<ReportArtifactDto> {
    return invoke("generate_report_artifacts", input);
}
```

```ts
type ReportState = {
    loading: boolean;
    error: string | null;
    latest: ReportArtifactDto | null;
    generate: (input: { reportName: string; outputDir: string }) => Promise<void>;
};
```

- [ ] **Step 4: Run frontend build again**

Run: `pnpm --filter desktop build`
Expected: PASS, with no mock-only type drift between Rust responses and TypeScript consumers.

- [ ] **Step 5: Commit**

```bash
git add apps/desktop/src/types/tauri-commands.ts apps/desktop/src/store/reportStore.ts apps/desktop/src/store/analysisStore.ts
git commit -m "feat: add desktop render report stores"
```

### Task 6: Replace report and chart mocks in the desktop UI

**Files:**
- Modify: `D:\Work\EtabExtension\apps\desktop\src\components\reports\ReportsPanel.tsx`
- Create: `D:\Work\EtabExtension\apps\desktop\src\components\reports\ReportArtifactsCard.tsx`
- Create: `D:\Work\EtabExtension\apps\desktop\src\components\analytics\RenderedChartFrame.tsx`
- Modify: `D:\Work\EtabExtension\apps\desktop\src\components\analytics\PerformanceChart.tsx`
- Test: `D:\Work\EtabExtension\apps\desktop\src\components\reports\ReportsPanel.tsx`

- [ ] **Step 1: Write the failing UI integration**

```tsx
import { useReportStore } from "@/store/reportStore";
import { ReportArtifactsCard } from "./ReportArtifactsCard";

const { latest, loading, error, generate } = useReportStore();
```

- [ ] **Step 2: Run the frontend build to verify it fails**

Run: `pnpm --filter desktop build`
Expected: FAIL until `ReportArtifactsCard`, the real store wiring, and the new report flow replace the mock-only implementation.

- [ ] **Step 3: Implement the real UI flow**

```tsx
const { loading, error, latest, generate } = useReportStore();

async function handleGenerate() {
    await generate({
        reportName: reportName.trim() || "structural-check-report",
        outputDir: selectedOutputDir,
    });
}
```

```tsx
export function RenderedChartFrame({ html }: { html: string }) {
    return <iframe srcDoc={html} className="h-[540px] w-full rounded-md border" title="Rendered chart" />;
}
```

- [ ] **Step 4: Verify the real UI compiles**

Run: `pnpm --filter desktop build`
Expected: PASS, with `ReportsPanel` using store-driven state and artifact cards instead of hard-coded mock data.

- [ ] **Step 5: Commit**

```bash
git add apps/desktop/src/components/reports/ReportsPanel.tsx apps/desktop/src/components/reports/ReportArtifactsCard.tsx apps/desktop/src/components/analytics/RenderedChartFrame.tsx apps/desktop/src/components/analytics/PerformanceChart.tsx
git commit -m "feat: wire desktop report ui"
```

### Task 7: Verification and documentation closeout

**Files:**
- Modify: `D:\Work\EtabExtension\crates\ext\skill\specs\handoff-week7-8.md`
- Modify: `D:\Work\EtabExtension\crates\ext\skill\specs\plan-week-7-8.md`
- Modify: `D:\Work\EtabExtension\crates\ext-render\EXT_RENDER_DESIGN.md`
- Modify: `D:\Work\EtabExtension\crates\ext-report\EXT_REPORT_DESIGN.md`

- [ ] **Step 1: Run Rust verification**

Run: `cargo check --target-dir .codex-target`
Expected: PASS for the workspace after the new render/report/tauri modules land.

- [ ] **Step 2: Run focused crate tests**

Run: `cargo test -p ext-api -p ext-render -p ext-report --target-dir .codex-target`
Expected: PASS, with workflow, render contract, and report artifact tests green.

- [ ] **Step 3: Run frontend verification**

Run: `pnpm --filter desktop build`
Expected: PASS, with no stale mock report UI or missing command types.

- [ ] **Step 4: Update handoff notes with verified commands**

```md
- `cargo check --target-dir .codex-target`
- `cargo test -p ext-api -p ext-render -p ext-report --target-dir .codex-target`
- `pnpm --filter desktop build`
```

- [ ] **Step 5: Commit**

```bash
git add crates/ext/skill/specs/handoff-week7-8.md crates/ext/skill/specs/plan-week-7-8.md crates/ext-render/EXT_RENDER_DESIGN.md crates/ext-report/EXT_REPORT_DESIGN.md
git commit -m "docs: close render report tauri handoff"
```
