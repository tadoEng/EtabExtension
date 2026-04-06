use ext_calc::output::BaseShearOutput;

use crate::chart_build::BASE_SHEAR_IMAGE;
use crate::chart_types::{CartesianSeries, ChartKind, ChartSpec, NamedChartSpec, RenderConfig, SeriesType};

pub fn build(base_shear: &BaseShearOutput, config: &RenderConfig) -> NamedChartSpec {
    NamedChartSpec {
        logical_name: BASE_SHEAR_IMAGE.to_string(),
        caption: "Equivalent static and response spectrum base shear comparison.".to_string(),
        spec: ChartSpec {
            title: "Base Shear Comparison".to_string(),
            width: config.width,
            height: config.height,
            kind: ChartKind::Cartesian {
                categories: vec!["X".to_string(), "Y".to_string()],
                series: vec![
                    CartesianSeries {
                        name: "RSA".to_string(),
                        data: vec![
                            base_shear.direction_x.v_rsa.value,
                            base_shear.direction_y.v_rsa.value,
                        ],
                        kind: SeriesType::Bar,
                    },
                    CartesianSeries {
                        name: "ELF".to_string(),
                        data: vec![
                            base_shear.direction_x.v_elf.value,
                            base_shear.direction_y.v_elf.value,
                        ],
                        kind: SeriesType::Bar,
                    },
                    CartesianSeries {
                        name: "Ratio".to_string(),
                        data: vec![base_shear.direction_x.ratio, base_shear.direction_y.ratio],
                        kind: SeriesType::Line,
                    },
                ],
            },
        },
    }
}
