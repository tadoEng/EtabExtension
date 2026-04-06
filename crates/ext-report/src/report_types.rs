#[derive(Debug, Clone, Default)]
pub struct ReportProjectMeta {
    pub project_name: String,
    pub project_number: String,
    pub reference: String,
    pub engineer: String,
    pub checker: String,
    pub date: String,
    pub subject: String,
    pub scale: String,
    pub revision: String,
    pub sheet_prefix: String,
}

#[derive(Debug, Clone)]
pub struct ChartRef {
    pub logical_name: String,
    pub caption: String,
}

#[derive(Debug, Clone)]
pub enum ChartLayout {
    SingleChart,
    TwoCharts,
    ChartAndTable,
    TableOnly,
}

#[derive(Debug, Clone, Default)]
pub struct KeyValueTable {
    pub title: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CalculationBlock {
    pub heading: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ReportSection {
    SummaryPage {
        title: String,
        lines: Vec<String>,
    },
    SingleChartPage {
        title: String,
        chart: ChartRef,
    },
    TwoChartsPage {
        title: String,
        charts: Vec<ChartRef>,
    },
    ChartAndTablePage {
        title: String,
        chart: ChartRef,
        table: KeyValueTable,
        table_emphasis: bool,
    },
    TableOnlyPage {
        title: String,
        table: KeyValueTable,
    },
    CalculationPage {
        title: String,
        blocks: Vec<CalculationBlock>,
    },
}

#[derive(Debug, Clone)]
pub struct ReportDocument {
    pub project: ReportProjectMeta,
    pub branch: String,
    pub version_id: String,
    pub overall_status: String,
    pub check_count: u32,
    pub pass_count: u32,
    pub fail_count: u32,
    pub sections: Vec<ReportSection>,
}
