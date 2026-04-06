pub mod pdf;
mod report_document;
mod report_types;

pub use pdf::{build_typst_document, render_pdf, write_pdf};
pub use report_document::build_report_document;
pub use report_types::{
    CalculationBlock, ChartLayout, ChartRef, KeyValueTable, ReportDocument, ReportProjectMeta,
    ReportSection,
};
