mod base_shear;
mod displacement;
mod drift;
mod modal;
mod pier_axial;
mod pier_shear;
mod story_forces;

use charming::{
    Chart,
    component::{Axis, Grid, Legend, Title},
    datatype::{CompositeValue, DataPoint},
    element::{AxisLabel, AxisType, Color, ItemStyle, LineStyle, LineStyleType, Tooltip, Trigger},
    series::{Bar, Line, Pie},
};
use ext_calc::output::CalcOutput;

use crate::chart_types::{
    ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

// ── Image path constants ────────────────────────────────────────────────────

pub const MODAL_IMAGE: &str = "images/modal.svg";
pub const BASE_REACTIONS_IMAGE: &str = "images/base_reactions.svg";

// Story forces — 4 assets
pub const STORY_FORCE_VX_IMAGE: &str = "images/story_force_vx.svg";
pub const STORY_FORCE_VY_IMAGE: &str = "images/story_force_vy.svg";
pub const STORY_FORCE_MY_IMAGE: &str = "images/story_force_my.svg";
pub const STORY_FORCE_MX_IMAGE: &str = "images/story_force_mx.svg";

// Drift — directional pairs
pub const DRIFT_WIND_X_IMAGE: &str = "images/drift_wind_x.svg";
pub const DRIFT_WIND_Y_IMAGE: &str = "images/drift_wind_y.svg";
pub const DRIFT_SEISMIC_X_IMAGE: &str = "images/drift_seismic_x.svg";
pub const DRIFT_SEISMIC_Y_IMAGE: &str = "images/drift_seismic_y.svg";

// Displacement — directional pairs
pub const DISPLACEMENT_WIND_X_IMAGE: &str = "images/displacement_wind_x.svg";
pub const DISPLACEMENT_WIND_Y_IMAGE: &str = "images/displacement_wind_y.svg";

// Pier shear
pub const PIER_SHEAR_STRESS_WIND_IMAGE: &str = "images/pier_shear_stress_wind.svg";
pub const PIER_SHEAR_STRESS_SEISMIC_IMAGE: &str = "images/pier_shear_stress_seismic.svg";

// Pier axial — 3 category assets (replaces the old single PIER_AXIAL_STRESS_IMAGE)
pub const PIER_AXIAL_GRAVITY_IMAGE: &str = "images/pier_axial_gravity.svg";
pub const PIER_AXIAL_WIND_IMAGE: &str = "images/pier_axial_wind.svg";
pub const PIER_AXIAL_SEISMIC_IMAGE: &str = "images/pier_axial_seismic.svg";

// ── Main builder ────────────────────────────────────────────────────────────

pub fn build_report_charts(calc: &CalcOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    let mut charts = Vec::new();

    if let Some(modal_output) = calc.modal.as_ref() {
        charts.push(modal::build(modal_output, config));
    }

    if let Some(base_reactions_output) = calc.base_reactions.as_ref() {
        charts.push(base_shear::build(base_reactions_output, config));
    }

    if let Some(sf_output) = calc.story_forces.as_ref() {
        charts.extend(story_forces::build(sf_output, config));
    }

    if let Some(drift_output) = calc.drift_wind.as_ref() {
        charts.extend(drift::build_wind(drift_output, config));
    }

    if let Some(drift_output) = calc.drift_seismic.as_ref() {
        charts.extend(drift::build_seismic(drift_output, config));
    }

    if let Some(displacement_output) = calc.displacement_wind.as_ref() {
        charts.extend(displacement::build(displacement_output, config));
    }

    if let Some(pier_output) = calc.pier_shear_stress_wind.as_ref() {
        charts.push(pier_shear::build_wind(pier_output, config));
    }

    if let Some(pier_output) = calc.pier_shear_stress_seismic.as_ref() {
        charts.push(pier_shear::build_seismic(pier_output, config));
    }

    if let Some(axial_output) = calc.pier_axial_stress.as_ref() {
        charts.extend(pier_axial::build_all(axial_output, config));
    }

    charts
}

// ── Chart rendering ─────────────────────────────────────────────────────────

pub fn build_chart(spec: &ChartSpec) -> Chart {
    match &spec.kind {
        ChartKind::Cartesian {
            categories,
            series,
            swap_axes,
        } => build_cartesian(spec, categories, series, *swap_axes),
        ChartKind::Pie { data } => build_pie(spec, data),
    }
}

fn build_cartesian(
    spec: &ChartSpec,
    categories: &[String],
    series: &[crate::chart_types::CartesianSeries],
    swap_axes: bool,
) -> Chart {
    let mut chart = Chart::new()
        .title(Title::new().text(spec.title.as_str()).left("center"))
        .grid(Grid::new().left("10%").right("6%").top("16%").bottom("16%"))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new().top("6%"))
        .x_axis(if swap_axes {
            Axis::new().type_(AxisType::Value)
        } else {
            Axis::new()
                .type_(AxisType::Category)
                .axis_label(AxisLabel::new().rotate(24))
                .data(categories.iter().map(String::as_str).collect::<Vec<_>>())
        })
        .y_axis(if swap_axes {
            Axis::new()
                .type_(AxisType::Category)
                .data(categories.iter().map(String::as_str).collect::<Vec<_>>())
        } else {
            Axis::new().type_(AxisType::Value)
        });

    for entry in series {
        chart = match entry.kind {
            SeriesType::Bar => {
                let data = if swap_axes {
                    build_swapped_axis_points(categories, &entry.data)
                } else {
                    entry.data.iter().copied().map(DataPoint::from).collect()
                };
                let mut series_builder = Bar::new().name(entry.name.as_str()).data(data);
                if let Some(color) = entry.color.as_deref() {
                    series_builder = series_builder
                        .item_style(ItemStyle::new().color(Color::Value(color.to_string())));
                }
                chart.series(series_builder)
            }
            SeriesType::Line => {
                let data = if swap_axes {
                    build_swapped_axis_points(categories, &entry.data)
                } else {
                    entry.data.iter().copied().map(DataPoint::from).collect()
                };
                let mut series_builder = Line::new()
                    .name(entry.name.as_str())
                    .smooth(entry.smooth)
                    .show_symbol(!matches!(entry.line_style, Some(LinePattern::Dashed)))
                    .data(data);
                if let Some(color) = entry.color.as_deref() {
                    series_builder = series_builder
                        .line_style(LineStyle::new().color(Color::Value(color.to_string())));
                }
                if let Some(pattern) = entry.line_style {
                    let mut style = LineStyle::new();
                    if let Some(color) = entry.color.as_deref() {
                        style = style.color(Color::Value(color.to_string()));
                    }
                    style = match pattern {
                        LinePattern::Solid => style.type_(LineStyleType::Solid),
                        LinePattern::Dashed => style.type_(LineStyleType::Dashed),
                    };
                    series_builder = series_builder.line_style(style);
                }
                chart.series(series_builder)
            }
        };
    }

    chart
}

fn build_pie(spec: &ChartSpec, data: &[(f64, String)]) -> Chart {
    Chart::new()
        .title(Title::new().text(spec.title.as_str()).left("center"))
        .tooltip(Tooltip::new().trigger(Trigger::Item))
        .legend(Legend::new().bottom("3%").left("center"))
        .series(
            Pie::new()
                .name(spec.title.as_str())
                .radius(vec!["35%", "65%"])
                .center(vec!["50%", "48%"])
                .data(
                    data.iter()
                        .map(|(value, label)| (*value, label.as_str()))
                        .collect::<Vec<_>>(),
                ),
        )
}

fn build_swapped_axis_points(categories: &[String], values: &[f64]) -> Vec<DataPoint> {
    categories
        .iter()
        .zip(values.iter())
        .map(|(category, value)| {
            DataPoint::from(vec![
                CompositeValue::from(*value),
                CompositeValue::from(category.clone()),
            ])
        })
        .collect()
}

/// Convert a `story_order` slice (top → bottom, as stored in `CalcOutput`) into the
/// display order expected by an ECharts swapped-axis category axis (bottom → top).
///
/// ECharts renders swapped-axis categories from bottom to top, so the first element
/// of the `categories` array ends up at the bottom of the chart.  Every chart builder
/// must therefore pass stories in ascending order (ground floor first, roof last).
///
/// # Arguments
/// * `story_order` – stories in top-down order (index 0 = roof, last = ground), as
///   populated by the calc layer from ETABS story definitions.
/// * `has_data` – predicate returning `true` when a story has at least one data point
///   and should appear on the axis.  Pass `|_| true` to keep all stories.
///
/// # Example
/// ```
/// let order = ["L3".to_string(), "L2".to_string(), "L1".to_string()];
/// let cats = story_display_order(&order, |_| true);
/// assert_eq!(cats, ["L1", "L2", "L3"]); // L1 at bottom, L3 at top
/// ```
pub fn story_display_order(
    story_order: &[String],
    has_data: impl Fn(&str) -> bool,
) -> Vec<String> {
    story_order
        .iter()
        .filter(|s| has_data(s.as_str()))
        .rev() // flip top→bottom into bottom→top so ECharts places them correctly
        .cloned()
        .collect()
}

pub(crate) fn is_default_pier_label(label: &str) -> bool {
    let trimmed = label.trim();
    trimmed.is_empty() || trimmed == "0"
}

pub(crate) fn normalized_pier_labels(labels: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = labels.into_iter().collect::<Vec<_>>();
    out.sort_by(|left, right| compare_pier_labels(left, right));
    out
}

fn compare_pier_labels(left: &str, right: &str) -> std::cmp::Ordering {
    let left_key = pier_label_key(left);
    let right_key = pier_label_key(right);
    left_key
        .0
        .cmp(&right_key.0)
        .then_with(|| left_key.1.cmp(&right_key.1))
        .then_with(|| natural_cmp(left, right))
}

fn pier_label_key(label: &str) -> (u8, u32) {
    if let Some(num) = parse_prefixed_number(label, "PX") {
        return (0, num);
    }
    if let Some(num) = parse_prefixed_number(label, "PY") {
        return (1, num);
    }
    (2, u32::MAX)
}

fn parse_prefixed_number(label: &str, prefix: &str) -> Option<u32> {
    let trimmed = label.trim();
    if !trimmed.to_ascii_uppercase().starts_with(prefix) {
        return None;
    }
    let suffix = &trimmed[prefix.len()..];
    if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    suffix.parse::<u32>().ok()
}

fn natural_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let mut li = left.chars().peekable();
    let mut ri = right.chars().peekable();

    loop {
        match (li.peek(), ri.peek()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(lc), Some(rc)) if lc.is_ascii_digit() && rc.is_ascii_digit() => {
                let mut l_num = String::new();
                let mut r_num = String::new();
                while let Some(ch) = li.peek() {
                    if ch.is_ascii_digit() {
                        l_num.push(*ch);
                        li.next();
                    } else {
                        break;
                    }
                }
                while let Some(ch) = ri.peek() {
                    if ch.is_ascii_digit() {
                        r_num.push(*ch);
                        ri.next();
                    } else {
                        break;
                    }
                }
                let l_val = l_num.parse::<u64>().unwrap_or(0);
                let r_val = r_num.parse::<u64>().unwrap_or(0);
                match l_val.cmp(&r_val) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            (Some(_), Some(_)) => {
                let l = li.next().unwrap().to_ascii_lowercase();
                let r = ri.next().unwrap().to_ascii_lowercase();
                match l.cmp(&r) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
        }
    }
}
