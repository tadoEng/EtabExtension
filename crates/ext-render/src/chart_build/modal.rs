use ext_calc::output::ModalOutput;

use crate::chart_build::MODAL_IMAGE;
use crate::chart_types::{
    CartesianSeries, ChartKind, ChartSpec, LinePattern, NamedChartSpec, RenderConfig, SeriesType,
};

pub fn build(modal: &ModalOutput, config: &RenderConfig) -> NamedChartSpec {
    NamedChartSpec {
        logical_name: MODAL_IMAGE.to_string(),
        caption: "Cumulative modal mass participation in the principal directions.".to_string(),
        spec: ChartSpec {
            title: "Modal Participation".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: modal.rows.iter().map(|row| format!("Mode {}", row.mode)).collect(),
                swap_axes: false,
                series: vec![
                    CartesianSeries {
                        name: "Sum UX".to_string(),
                        data: modal.rows.iter().map(|row| row.sum_ux).collect(),
                        kind: SeriesType::Line,
                        color: Some("#1f77b4".to_string()),
                        line_style: Some(LinePattern::Solid),
                        smooth: true,
                    },
                    CartesianSeries {
                        name: "Sum UY".to_string(),
                        data: modal.rows.iter().map(|row| row.sum_uy).collect(),
                        kind: SeriesType::Line,
                        color: Some("#f28e2b".to_string()),
                        line_style: Some(LinePattern::Solid),
                        smooth: true,
                    },
                ],
            },
        },
    }
}
