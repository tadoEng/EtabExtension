pub mod data;
pub mod pdf;
pub mod theme;

pub use data::{ReportData, ReportProjectMeta};
pub use pdf::{build_typst_document, render_pdf, write_pdf};
pub use theme::{A4_PORTRAIT, PageTheme, TABLOID_LANDSCAPE};
