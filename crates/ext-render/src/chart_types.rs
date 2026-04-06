#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 900,
            height: 620,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChartSpec {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub kind: ChartKind,
}

#[derive(Debug, Clone)]
pub enum ChartKind {
    Cartesian {
        categories: Vec<String>,
        series: Vec<CartesianSeries>,
    },
    Pie {
        data: Vec<(f64, String)>,
    },
}

#[derive(Debug, Clone)]
pub struct CartesianSeries {
    pub name: String,
    pub data: Vec<f64>,
    pub kind: SeriesType,
}

#[derive(Debug, Clone, Copy)]
pub enum SeriesType {
    Bar,
    Line,
}

#[derive(Debug, Clone)]
pub struct NamedChartSpec {
    pub logical_name: String,
    pub caption: String,
    pub spec: ChartSpec,
}

#[derive(Debug, Clone)]
pub struct RenderedAsset {
    pub logical_name: String,
    pub caption: String,
    pub svg: String,
}

#[derive(Debug, Clone, Default)]
pub struct RenderedCharts {
    pub assets: Vec<RenderedAsset>,
}
