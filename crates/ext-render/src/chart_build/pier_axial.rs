use std::collections::HashMap;

use ext_calc::output::PierAxialOutput;

use crate::chart_build::PIER_AXIAL_IMAGE;
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build(output: &PierAxialOutput, config: &RenderConfig) -> NamedChartSpec {
    // Collect all stories in their natural order (first seen = bottom to top).
    let mut stories: Vec<String> = Vec::new();
    for row in &output.piers {
        if !stories.contains(&row.story) {
            stories.push(row.story.clone());
        }
    }

    // Group values by pier label: pier → { story → fa (ksi) }
    let mut pier_map: Vec<(String, HashMap<String, f64>)> = Vec::new();
    for row in &output.piers {
        if let Some((_, map)) = pier_map.iter_mut().find(|(label, _)| label == &row.pier_label) {
            // Keep the governing (max absolute) value per story per pier.
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

    // Build one Line series per pier.
    let palette = [
        "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728",
        "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
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

    // Zero-reference dashed vertical line (x = 0 for every story).
    series.push(CartesianSeries {
        name: "Zero".to_string(),
        data: vec![0.0; stories.len()],
        kind: SeriesType::Line,
        color: Some("#aaaaaa".to_string()),
        line_style: Some(LinePattern::Dashed),
        smooth: false,
    });

    NamedChartSpec {
        logical_name: PIER_AXIAL_IMAGE.to_string(),
        caption: "Signed axial stress envelope per pier (ksi). Dashed line = zero.".to_string(),
        spec: ChartSpec {
            title: "Pier Axial Stress".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: stories,
                swap_axes: true, // Y = story, X = stress (ksi)
                series,
            },
        },
    }
}
