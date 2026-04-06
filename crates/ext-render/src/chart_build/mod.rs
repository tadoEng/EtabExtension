mod base_shear;
mod displacement;
mod drift;
mod modal;
mod pier_axial;
mod pier_shear;

use charming::{
    Chart,
    component::{Axis, Grid, Legend, Title},
    element::{AxisType, Tooltip, Trigger},
    series::{Bar, Line, Pie},
};
use ext_calc::output::CalcOutput;

use crate::chart_types::{ChartKind, ChartSpec, NamedChartSpec, RenderConfig, SeriesType};

pub const MODAL_IMAGE: &str = "images/modal.svg";
pub const BASE_SHEAR_IMAGE: &str = "images/base_shear.svg";
pub const DRIFT_WIND_IMAGE: &str = "images/drift_wind.svg";
pub const DRIFT_SEISMIC_IMAGE: &str = "images/drift_seismic.svg";
pub const DISPLACEMENT_WIND_IMAGE: &str = "images/displacement_wind.svg";
pub const PIER_SHEAR_WIND_IMAGE: &str = "images/pier_shear_wind.svg";
pub const PIER_SHEAR_SEISMIC_IMAGE: &str = "images/pier_shear_seismic.svg";
pub const PIER_AXIAL_IMAGE: &str = "images/pier_axial.svg";

pub fn build_report_charts(calc: &CalcOutput, config: &RenderConfig) -> Vec<NamedChartSpec> {
    let mut charts = Vec::new();

    if let Some(modal_output) = calc.modal.as_ref() {
        charts.push(modal::build(modal_output, config));
    }

    if let Some(base_shear_output) = calc.base_shear.as_ref() {
        charts.push(base_shear::build(base_shear_output, config));
    }

    if let Some(drift_output) = calc.drift_wind.as_ref() {
        charts.push(drift::build_wind(drift_output, config));
    }

    if let Some(drift_output) = calc.drift_seismic.as_ref() {
        charts.push(drift::build_seismic(drift_output, config));
    }

    if let Some(displacement_output) = calc.displacement_wind.as_ref() {
        charts.push(displacement::build(displacement_output, config));
    }

    if let Some(pier_output) = calc.pier_shear_wind.as_ref() {
        charts.push(pier_shear::build_wind(pier_output, config));
    }

    if let Some(pier_output) = calc.pier_shear_seismic.as_ref() {
        charts.push(pier_shear::build_seismic(pier_output, config));
    }

    if let Some(axial_output) = calc.pier_axial.as_ref() {
        charts.push(pier_axial::build(axial_output, config));
    }

    charts
}

pub fn build_chart(spec: &ChartSpec) -> Chart {
    match &spec.kind {
        ChartKind::Cartesian { categories, series } => build_cartesian(spec, categories, series),
        ChartKind::Pie { data } => build_pie(spec, data),
    }
}

fn build_cartesian(
    spec: &ChartSpec,
    categories: &[String],
    series: &[crate::chart_types::CartesianSeries],
) -> Chart {
    let mut chart = Chart::new()
        .title(Title::new().text(spec.title.as_str()).left("center"))
        .grid(
            Grid::new()
                .left("10%")
                .right("6%")
                .top("16%")
                .bottom("16%"),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .legend(Legend::new().top("6%"))
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(categories.iter().map(String::as_str).collect::<Vec<_>>()),
        )
        .y_axis(Axis::new().type_(AxisType::Value));

    for entry in series {
        chart = match entry.kind {
            SeriesType::Bar => chart.series(Bar::new().name(entry.name.as_str()).data(entry.data.clone())),
            SeriesType::Line => chart.series(
                Line::new()
                    .name(entry.name.as_str())
                    .smooth(true)
                    .data(entry.data.clone()),
            ),
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

pub(crate) fn top_pier_values(iter: impl Iterator<Item = (String, f64)>) -> Vec<(String, f64)> {
    let mut values = iter.collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.total_cmp(&left.1));
    values.truncate(8);
    values.reverse();
    values
}
