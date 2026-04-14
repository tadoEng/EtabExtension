use std::collections::HashMap;

use ext_calc::output::PierAxialStressOutput;

use crate::chart_build::{
    PIER_AXIAL_GRAVITY_IMAGE, PIER_AXIAL_SEISMIC_IMAGE, PIER_AXIAL_WIND_IMAGE,
};
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

/// Build 3 pier axial stress charts: gravity, wind, and seismic categories.
/// Returns only assets for categories that have at least one result row.
pub fn build_all(output: &PierAxialStressOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    let mut charts = Vec::new();
    if let Some(chart) = build_category(output, config, "gravity", PIER_AXIAL_GRAVITY_IMAGE, "Pier Axial — Gravity", "Signed axial stress envelope per pier under gravity combos (ksi).") {
        charts.push(chart);
    }
    if let Some(chart) = build_category(output, config, "wind", PIER_AXIAL_WIND_IMAGE, "Pier Axial — Wind", "Signed axial stress envelope per pier under wind combos (ksi).") {
        charts.push(chart);
    }
    if let Some(chart) = build_category(output, config, "seismic", PIER_AXIAL_SEISMIC_IMAGE, "Pier Axial — Seismic", "Signed axial stress envelope per pier under seismic combos (ksi).") {
        charts.push(chart);
    }
    charts
}

fn build_category(
    output: &PierAxialStressOutput,
    config: &RenderConfig,
    category: &str,
    logical_name: &str,
    title: &str,
    caption: &str,
) -> Option<NamedChartSpec> {
    let filtered: Vec<_> = output
        .piers
        .iter()
        .filter(|r| r.category == category)
        .collect();

    if filtered.is_empty() {
        return None;
    }

    // Collect all stories in natural order (first seen).
    let mut stories: Vec<String> = Vec::new();
    for row in &filtered {
        if !stories.contains(&row.story) {
            stories.push(row.story.clone());
        }
    }

    // Group values by pier label: pier → { story → fa (ksi) }
    // Keep the governing (max absolute) value per story per pier.
    let mut pier_map: Vec<(String, HashMap<String, f64>)> = Vec::new();
    for row in &filtered {
        if let Some((_, map)) = pier_map
            .iter_mut()
            .find(|(label, _)| label == &row.pier_label)
        {
            let entry = map.entry(row.story.clone()).or_insert(row.fa.value);
            if row.fa.value.abs() > entry.abs() {
                *entry = row.fa.value;
            }
        } else {
            let mut map = HashMap::new();
            map.insert(row.story.clone(), row.fa.value);
            pier_map.push((row.pier_label.clone(), map));
        }
    }

    let palette = [
        "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
    ];
    let mut series: Vec<CartesianSeries> = pier_map
        .iter()
        .enumerate()
        .map(|(idx, (pier_label, val_map))| {
            let data = stories
                .iter()
                .map(|s| *val_map.get(s).unwrap_or(&f64::NAN))
                .collect();
            CartesianSeries {
                name: pier_label.clone(),
                data,
                kind: SeriesType::Line,
                color: Some(palette[idx % palette.len()].to_string()),
                line_style: Some(LinePattern::Solid),
                smooth: false,
            }
        })
        .collect();

    // Zero-reference dashed line.
    series.push(CartesianSeries {
        name: "Zero".to_string(),
        data: vec![0.0; stories.len()],
        kind: SeriesType::Line,
        color: Some("#aaaaaa".to_string()),
        line_style: Some(LinePattern::Dashed),
        smooth: false,
    });

    let story_count = stories.len() as u32;
    let height = config.height.max(story_count * 18 + 100);

    Some(NamedChartSpec {
        logical_name: logical_name.to_string(),
        caption: caption.to_string(),
        spec: ChartSpec {
            title: title.to_string(),
            width: config.width,
            height,
            kind: ChartKind::Cartesian {
                categories: stories,
                swap_axes: true, // Y = story, X = stress (ksi)
                series,
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use ext_calc::output::{PierAxialResult, PierAxialStressOutput, Quantity};
    use crate::chart_build::{
        PIER_AXIAL_GRAVITY_IMAGE, PIER_AXIAL_SEISMIC_IMAGE, PIER_AXIAL_WIND_IMAGE,
    };
    use crate::chart_types::RenderConfig;
    use super::build_all;

    fn make_result(pier: &str, story: &str, category: &str, fa_ksi: f64) -> PierAxialResult {
        PierAxialResult {
            pier_label: pier.to_string(),
            story: story.to_string(),
            combo: format!("{}-combo", category),
            category: category.to_string(),
            pu: Quantity::new(100.0, "kip"),
            ag: Quantity::new(144.0, "in²"),
            phi_po: Quantity::new(500.0, "kip"),
            fa: Quantity::new(fa_ksi, "ksi"),
            fa_ratio: fa_ksi / (0.85 * 4.0),
            dcr: fa_ksi / (0.85 * 4.0),
            pass: true,
            fc_ksi: 4.0,
            material: "C4000".to_string(),
        }
    }

    fn dummy_governing() -> PierAxialResult {
        make_result("P1", "L1", "gravity", 0.5)
    }

    #[test]
    fn pier_axial_build_all_returns_three_category_assets() {
        let output = PierAxialStressOutput {
            phi_axial: 0.65,
            piers: vec![
                make_result("P1", "L1", "gravity", 0.5),
                make_result("P1", "L2", "gravity", 0.4),
                make_result("P1", "L1", "wind", 0.3),
                make_result("P1", "L1", "seismic", 0.2),
            ],
            governing_gravity: Some(make_result("P1", "L1", "gravity", 0.5)),
            governing_wind: Some(make_result("P1", "L1", "wind", 0.3)),
            governing_seismic: Some(make_result("P1", "L1", "seismic", 0.2)),
            governing: dummy_governing(),
            pass: true,
        };

        let config = RenderConfig::default();
        let charts = build_all(&output, &config);

        assert_eq!(charts.len(), 3, "expected 3 category chart assets");

        let names: Vec<&str> = charts.iter().map(|c| c.logical_name.as_str()).collect();
        assert!(names.contains(&PIER_AXIAL_GRAVITY_IMAGE), "missing gravity chart");
        assert!(names.contains(&PIER_AXIAL_WIND_IMAGE), "missing wind chart");
        assert!(names.contains(&PIER_AXIAL_SEISMIC_IMAGE), "missing seismic chart");
    }

    #[test]
    fn pier_axial_omits_missing_category() {
        // Only gravity rows — wind and seismic charts must be absent.
        let output = PierAxialStressOutput {
            phi_axial: 0.65,
            piers: vec![
                make_result("P1", "L1", "gravity", 0.5),
            ],
            governing_gravity: Some(make_result("P1", "L1", "gravity", 0.5)),
            governing_wind: None,
            governing_seismic: None,
            governing: dummy_governing(),
            pass: true,
        };

        let charts = build_all(&output, &RenderConfig::default());
        assert_eq!(charts.len(), 1, "only gravity rows → only 1 chart");
        assert_eq!(charts[0].logical_name, PIER_AXIAL_GRAVITY_IMAGE);
    }

    #[test]
    fn no_torsional_chart_in_build_all() {
        // build_all should never return a chart whose logical_name contains "torsional".
        let output = PierAxialStressOutput {
            phi_axial: 0.65,
            piers: vec![make_result("P1", "L1", "gravity", 0.5)],
            governing_gravity: Some(make_result("P1", "L1", "gravity", 0.5)),
            governing_wind: None,
            governing_seismic: None,
            governing: dummy_governing(),
            pass: true,
        };
        let charts = build_all(&output, &RenderConfig::default());
        for chart in &charts {
            assert!(
                !chart.logical_name.contains("torsional"),
                "unexpected torsional chart: {}",
                chart.logical_name
            );
        }
    }
}
