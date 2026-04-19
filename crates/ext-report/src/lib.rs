pub mod data;
pub mod pdf;
pub mod theme;

pub use data::{ReportData, ReportProjectMeta};
pub use pdf::{
    RenderPdfOptions, build_typst_document, render_pdf, render_pdf_with_options, write_pdf,
};
pub use theme::{A4_PORTRAIT, PageTheme, ReportTheme, TABLOID_LANDSCAPE};
