# ECharts SSR with Typst Report Generator

A Rust-only application that renders ECharts visualizations server-side and embeds them into professional PDF reports generated using Typst. No Node.js, no JavaScript runtime, no subprocesses.

## Features

- **Pure Rust pipeline**: Charts rendered via [charming](https://github.com/yuankunzhang/charming), which embeds ECharts through a bundled Deno engine — no external runtime required
- **In-memory SVG handoff**: SVG bytes flow directly from charming into Typst's world without touching disk
- **Typst integration**: Professional multi-page PDF reports with embedded charts, calculations, and DCR tables
- **CLI interface**: Single binary, two commands
- **Multi-target**: Same chart code works for CLI (SSR feature), Tauri webview (HTML renderer), and WASM (wasm feature)

## Project structure

```
echart_ssr/
├── Cargo.toml          # Rust dependencies and feature flags
├── src/
│   ├── main.rs         # CLI entry point
│   ├── chart.rs        # ChartSpec → charming → SVG string
│   └── typst.rs        # SVG bytes → Typst world → PDF bytes
└── target/
    └── release/
        └── echart_ssr  # Compiled binary (no sidecar files needed)
```

## Prerequisites

- Rust (latest stable)

That is the entire list.

## Installation

```bash
cargo build --release --features ssr
```

The `ssr` feature bundles a Deno engine inside the binary so charming can execute ECharts JavaScript internally. This adds roughly 50–80 MB to the binary but eliminates all runtime dependencies.

## CLI commands

### Generate a standalone SVG chart

```bash
./target/release/echart_ssr chart [OUTPUT_FILE]
```

```bash
# default output: chart.svg
./target/release/echart_ssr chart

# custom path
./target/release/echart_ssr chart analysis.svg
```

### Generate a PDF report

```bash
./target/release/echart_ssr report [OUTPUT_FILE]
```

```bash
# default output: report.pdf
./target/release/echart_ssr report

# custom path
./target/release/echart_ssr report analysis_report.pdf
```

### Help

```bash
./target/release/echart_ssr --help
```

## Architecture

### Data flow

```
ChartSpec
  → build_chart()              pure Rust, no I/O
  → ImageRenderer::render()    charming/Deno executes ECharts → SVG String
  → svg.into_bytes()           String → Vec<u8>, zero-copy conversion
  → TypstWorld image_cache     injected as virtual file, never written to disk
  → typst::compile()           produces Vec<u8> PDF bytes
  → fs::write()                single disk write at the very end
```

The only file written to disk is the final PDF (or SVG when using the `chart` command). All intermediate data stays in memory.

### Chart module (`src/chart.rs`)

Defines `ChartSpec` and converts it to a charming `Chart`:

```rust
pub struct ChartSpec {
    pub title:  String,
    pub x:      Vec<String>,
    pub series: Vec<Series>,
    pub width:  u32,
    pub height: u32,
}

pub enum SeriesType { Bar, Line }
```

Three renderer functions, each gated appropriately:

| Function | Feature flag | Use case |
|---|---|---|
| `render_svg()` | `ssr` | CLI, Tauri sidecar, server |
| `render_html()` | none | Tauri webview (browser renders ECharts) |
| `render_wasm()` | `wasm` | Tauri WASM frontend |

### Typst module (`src/typst.rs`)

`TypstWorld` implements `typst::World`. Its `file()` method serves image bytes from an in-memory `HashMap<PathBuf, Bytes>` that is pre-populated with SVG output from charming before compilation starts:

```rust
// SVG string from charming injected directly — no disk write
let svg_bytes = Bytes::new(svg_string.into_bytes());
extra_images.insert(PathBuf::from("images/chart.svg"), svg_bytes);

// Typst calls World::file("images/chart.svg") → hits the cache
let pdf = compile_to_pdf(&typst_content, extra_images)?;
```

Report structure:
- Page 1: Cover with project information
- Page 2: Embedded chart image(s)
- Page 3: Design calculations
- Page 4+: DCR analysis tables (paginated automatically)

### Feature flags

```toml
[features]
ssr  = ["charming/ssr"]   # CLI / sidecar — bundles Deno
wasm = ["charming/wasm"]  # Tauri WASM frontend — mutually exclusive with ssr
```

The base crate (no features) always exposes `build_chart()` and `render_html()`, which is enough for Tauri webview usage where the browser handles ECharts rendering.

## Customizing charts

Edit `force_displacement_spec()` in `main.rs`, or call `chart::build_chart()` directly with your own `ChartSpec`:

```rust
let spec = chart::ChartSpec {
    title:  "My Chart".into(),
    x:      vec!["Jan".into(), "Feb".into(), "Mar".into()],
    series: vec![
        chart::Series {
            name: "Revenue".into(),
            data: vec![120.0, 95.0, 140.0],
            kind: chart::SeriesType::Bar,
        },
    ],
    width:  800,
    height: 600,
};

let svg = chart::render_svg(&spec)?;
```

## Customizing reports

Call `typst::generate_report_from_svg()` with your own data, or use `generate_report_from_svgs()` to embed multiple charts in a single report:

```rust
// Multiple charts in one report
let mut svgs = HashMap::new();
svgs.insert("images/forces.svg".into(),       chart::render_svg(&force_spec)?);
svgs.insert("images/displacement.svg".into(), chart::render_svg(&disp_spec)?);

typst::generate_report_from_svgs("report.pdf", report_data, svgs)?;
```

## Output

| Output | Format | Typical size |
|---|---|---|
| Standalone chart | SVG vector | ~6–8 KB |
| Single-chart report | PDF | ~130–140 KB |

## Building from source

```bash
# Type-check only (no Deno bundled, fast)
cargo check

# Debug build
cargo build --features ssr

# Optimized release build
cargo build --release --features ssr

# Run tests
cargo test

# Clean
cargo clean
```

## Troubleshooting

### Font loading issues

**Error**: `no fonts could be loaded`

Fonts are auto-detected from `C:\Windows\Fonts` on Windows and from a `fonts/` directory in the working directory. If neither exists, add a `fonts/` directory containing at least one `.ttf` or `.otf` file.

### Image not found in PDF

**Error**: Typst reports a file not found for an image path

The logical filename passed to `generate_report_from_svg()` must exactly match the path used in the Typst markup, including any `images/` prefix. For example, if the markup contains `image("images/chart.svg")`, the logical name must be `"images/chart.svg"`.

### Binary too large

The `ssr` feature bundles Deno and is expected to produce a large binary (~80–100 MB release). This is normal. If binary size is a concern for Tauri distribution, use `render_html()` (no feature flag) and let the Tauri webview render ECharts — the binary will be significantly smaller.

## Further reading

- [charming](https://github.com/yuankunzhang/charming) — Rust ECharts wrapper
- [Typst documentation](https://typst.app/docs/)
- [ECharts SSR handbook](https://echarts.apache.org/handbook/en/how-to/cross-platform/server/)
- [Rust Lang Book](https://doc.rust-lang.org/book/)