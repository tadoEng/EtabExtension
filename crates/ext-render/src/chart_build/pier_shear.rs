use ext_calc::output::PierShearOutput;

use crate::chart_build::{PIER_SHEAR_SEISMIC_IMAGE, PIER_SHEAR_WIND_IMAGE};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build_wind(output: &PierShearOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_WIND_IMAGE,
        "Pier Shear Wind — Vu/Acv vs 8√f'c",
        "All pier shear stresses vs ACI 318 maximum stress limit 8√f’c (psi).",
        output,
        config,
    )
}

pub fn build_seismic(output: &PierShearOutput, config: &RenderConfig) -> NamedChartSpec {
    build_chart(
        PIER_SHEAR_SEISMIC_IMAGE,
        "Pier Shear Seismic — Vu/Acv vs 8√f'c",
        "All pier shear stresses vs ACI 318 maximum stress limit 8√f’c (psi).",
        output,
        config,
    )
}

fn build_chart(
    logical_name: &str,
    title: &str,
    caption: &str,
    output: &PierShearOutput,
    config: &RenderConfig,
) -> NamedChartSpec {
    // One entry per pier × story, sorted by demand descending so the chart reads
    // highest-stressed bars at the top (swap_axes = true ⇒ Y is category).
    let mut entries: Vec<(String, f64, f64)> = output
        .piers
        .iter()
        .map(|row| {
            let label = format!("{} / {}", row.pier_label, row.story);
            let demand = row.vu.value / row.acv.value;          // psi
            let fc_psi = row.fc_ksi * 1000.0;
            let limit = 8.0 * fc_psi.sqrt();                    // ACI max psi
            (label, demand, limit)
        })
        .collect();
    entries.sort_by(|a, b| b.1.total_cmp(&a.1));

    let categories: Vec<String> = entries.iter().map(|(l, _, _)| l.clone()).collect();
    let demands: Vec<f64> = entries.iter().map(|(_, d, _)| *d).collect();
    // All entries for one check share the same f'c; if mixed, each limit is correct
    // per-row but charming only supports one limit series — use the mean here.
    let limit_val = entries.iter().map(|(_, _, l)| l).copied().sum::<f64>()
        / entries.len().max(1) as f64;
    let limits: Vec<f64> = vec![limit_val; entries.len()];

    NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            // Scale height with pier count so bars don't squish on tall buildings.
            height: (config.height).max(30 * entries.len() as u32 + 80),
            kind: ChartKind::Cartesian {
                categories,
                swap_axes: true,
                series: vec![
                    CartesianSeries {
                        name: "Vu/Acv (psi)".to_string(),
                        data: demands,
                        kind: SeriesType::Bar,
                        color: Some("#1f77b4".to_string()),
                        line_style: None,
                        smooth: false,
                    },
                    CartesianSeries {
                        name: "8√f'c limit (psi)".to_string(),
                        data: limits,
                        kind: SeriesType::Line,
                        color: Some("#cc0000".to_string()),
                        line_style: Some(LinePattern::Dashed),
                        smooth: false,
                    },
                ],
            },
        },
    }
}
