pub mod chart;
pub mod typst;

use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let result = match args[1].as_str() {
        "chart"         => cmd_chart(&args),
        "report"        => cmd_report(&args),
        "--help" | "-h" => { print_help(); Ok(()) }
        other => {
            eprintln!("unknown command: {other}");
            print_help();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn print_help() {
    println!("echart_ssr chart  [OUTPUT.svg]  — render sample SVG chart to file");
    println!("echart_ssr report [OUTPUT.pdf]  — render full multi-page PDF report");
}

// ─── chart command ────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn cmd_chart(args: &[String]) -> anyhow::Result<()> {
    let out = args.get(2).map(String::as_str).unwrap_or("chart.svg");
    let svg = chart::render_svg(&force_disp_spec())?;
    std::fs::write(out, &svg)?;
    println!("chart saved: {out}");
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn cmd_chart(_: &[String]) -> anyhow::Result<()> {
    anyhow::bail!("build with --features ssr to use the chart command")
}

// ─── report command ───────────────────────────────────────────────────────────
//
// Full in-memory pipeline:
//   1. Render each ChartSpec → SVG String via charming (no disk write)
//   2. Collect into HashMap<logical_name, svg_string>
//   3. Pass to generate_report() which injects bytes into TypstWorld cache
//   4. Typst compiles → PDF bytes → single fs::write at the end

#[cfg(feature = "ssr")]
fn cmd_report(args: &[String]) -> anyhow::Result<()> {
    let out = args.get(2).map(String::as_str).unwrap_or("report.pdf");

    println!("rendering charts...");
    let mut svgs: HashMap<String, String> = HashMap::new();

    // Chart 1: Base reactions pie
    let pie = chart::base_reaction_pie(2450.0, 820.0, 310.0, 178.0, 240.0);
    svgs.insert("images/base_reactions.svg".into(), chart::render_svg(&pie)?);
    println!("  base_reactions.svg done");

    // Chart 2: Force vs Displacement bar+line
    let fd = force_disp_spec();
    svgs.insert("images/force_disp.svg".into(), chart::render_svg(&fd)?);
    println!("  force_disp.svg done");

    // Chart 3: Story shear line chart
    let stories = vec![
        "Roof","L10","L09","L08","L07","L06","L05","L04","L03","L02","L01",
    ].iter().map(|s| s.to_string()).collect();
    // Shear accumulates toward base — realistic profile
    let x_shear = vec![24.0, 58.0, 88.0, 114.0, 138.0, 160.0, 204.0, 246.0, 282.0, 312.0, 336.0];
    let y_shear = vec![22.0, 54.0, 82.0, 108.0, 130.0, 150.0, 192.0, 230.0, 264.0, 292.0, 314.0];
    let ss = chart::story_shear_chart(stories, x_shear, y_shear);
    svgs.insert("images/story_shear.svg".into(), chart::render_svg(&ss)?);
    println!("  story_shear.svg done");

    // Chart 4: Wind drift envelope
    let drift_stories = vec![
        "Roof","L10","L09","L08","L07","L06","L05","L04","L03","L02","L01",
    ].iter().map(|s| s.to_string()).collect();
    let drift_demand = vec![0.012, 0.011, 0.010, 0.009, 0.008, 0.007, 0.008, 0.009, 0.010, 0.011, 0.012];
    let drift = chart::drift_envelope_chart(drift_stories, drift_demand, 0.025);
    svgs.insert("images/drift_wind.svg".into(), chart::render_svg(&drift)?);
    println!("  drift_wind.svg done");

    // Build report data
    let project = typst::ProjectData {
        project_name: "Pacific Tower".into(),
        project_num:  "2025-0042".into(),
        reference:    "S-FOUND-01".into(),
        engineer:     "BT".into(),
        checker:      "RS".into(),
        date:         chrono::Local::now().format("%Y-%m-%d").to_string(),
        subject:      "Foundation Design — Gravity & Lateral".into(),
        scale:        "NTS".into(),
        sheet:        "SK-01".into(),
        revision:     "0".into(),
    };

    let data = typst::example_report_data(project);

    println!("compiling PDF...");
    typst::generate_report(out, &data, svgs)?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn cmd_report(_: &[String]) -> anyhow::Result<()> {
    anyhow::bail!("build with --features ssr to use the report command")
}

// ─── Chart spec factories ─────────────────────────────────────────────────────

fn force_disp_spec() -> chart::ChartSpec {
    chart::force_displacement_chart(
        ["A", "B", "C", "D"].iter().map(|s| s.to_string()).collect(),
        vec![10.0, 20.0, 15.0, 30.0],
        vec![5.0,  15.0, 25.0, 10.0],
    )
}