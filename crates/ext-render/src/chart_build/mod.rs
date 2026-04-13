mod base_shear;
mod displacement;
mod drift;
mod modal;
mod pier_axial;
mod pier_shear;

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

pub const MODAL_IMAGE: &str = "images/modal.svg";
pub const BASE_REACTIONS_IMAGE: &str = "images/base_reactions.svg";
pub const STORY_FORCE_VX_IMAGE: &str = "images/story_force_vx.svg";
pub const STORY_FORCE_VY_IMAGE: &str = "images/story_force_vy.svg";
pub const DRIFT_WIND_X_IMAGE: &str = "images/drift_wind_x.svg";
pub const DRIFT_WIND_Y_IMAGE: &str = "images/drift_wind_y.svg";
pub const DRIFT_SEISMIC_X_IMAGE: &str = "images/drift_seismic_x.svg";
pub const DRIFT_SEISMIC_Y_IMAGE: &str = "images/drift_seismic_y.svg";
pub const DISPLACEMENT_WIND_X_IMAGE: &str = "images/displacement_wind_x.svg";
pub const DISPLACEMENT_WIND_Y_IMAGE: &str = "images/displacement_wind_y.svg";
pub const PIER_SHEAR_STRESS_WIND_IMAGE: &str = "images/pier_shear_stress_wind.svg";
pub const PIER_SHEAR_STRESS_SEISMIC_IMAGE: &str = "images/pier_shear_stress_seismic.svg";
pub const PIER_AXIAL_STRESS_IMAGE: &str = "images/pier_axial_stress.svg";

pub fn build_report_charts(calc: &CalcOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    let mut charts = Vec::new();

    if let Some(modal_output) = calc.modal.as_ref() {
        charts.push(modal::build(modal_output, config));
    }

    if let Some(base_reactions_output) = calc.base_reactions.as_ref() {
        charts.push(base_shear::build(base_reactions_output, config));
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
        charts.push(pier_axial::build(axial_output, config));
    }

    charts
}

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

pub(crate) fn aggregate_story_max(iter: impl Iterator<Item = (String, f64)>) -> Vec<(String, f64)> {
    let mut values: Vec<(String, f64)> = Vec::new();

    for (story, value) in iter {
        if let Some((_, existing)) = values.iter_mut().find(|(name, _)| *name == story) {
            *existing = existing.max(value);
        } else {
            values.push((story, value));
        }
    }

    values
}
