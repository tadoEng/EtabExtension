use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_pdf::PdfOptions;

use crate::pdf::template::build_typst_document;
use crate::report_types::{ReportDocument, ReportSection};

struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
    image_cache: HashMap<PathBuf, Bytes>,
}

impl TypstWorld {
    fn new(content: String, images: HashMap<PathBuf, Bytes>) -> Result<Self> {
        let mut fonts = Vec::new();
        let mut book = FontBook::new();

        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::search_fonts(&current_dir.join("fonts"), &mut fonts, &mut book);
        Self::search_fonts(Path::new(r"C:\Windows\Fonts"), &mut fonts, &mut book);

        if fonts.is_empty() {
            bail!("No readable fonts found for Typst compilation");
        }

        Ok(Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
            main: Source::new(
                FileId::new(None, VirtualPath::new("main.typ")),
                content,
            ),
            image_cache: images,
        })
    }

    fn search_fonts(path: &Path, fonts: &mut Vec<Font>, book: &mut FontBook) {
        if !path.exists() {
            return;
        }

        for entry in walkdir::WalkDir::new(path)
            .follow_links(true)
            .sort_by(|left, right| left.file_name().cmp(right.file_name()))
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            let font_path = entry.path();
            if !font_path.is_file() {
                continue;
            }

            let extension = font_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if !matches!(extension.as_str(), "ttf" | "otf" | "ttc" | "otc") {
                continue;
            }

            let Ok(buffer) = fs::read(font_path) else {
                continue;
            };
            let bytes = Bytes::new(buffer);
            for font in Font::iter(bytes) {
                book.push(font.info().clone());
                fonts.push(font);
            }
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rootless_path().into(),
            ))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = Path::new(id.vpath().as_rootless_path());
        if let Some(bytes) = self.image_cache.get(path) {
            return Ok(bytes.clone());
        }
        Err(typst::diag::FileError::NotFound(path.to_path_buf()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _: Option<i64>) -> Option<Datetime> {
        None
    }
}

pub fn render_pdf(document: &ReportDocument, svg_map: HashMap<String, String>) -> Result<Vec<u8>> {
    for section in &document.sections {
        match section {
            ReportSection::SingleChartPage { chart, .. }
            | ReportSection::ChartAndTablePage { chart, .. } => {
                if !svg_map.contains_key(&chart.logical_name) {
                    bail!("Missing SVG asset for '{}'", chart.logical_name);
                }
            }
            ReportSection::TwoChartsPage { charts, .. } => {
                for chart in charts {
                    if !svg_map.contains_key(&chart.logical_name) {
                        bail!("Missing SVG asset for '{}'", chart.logical_name);
                    }
                }
            }
            ReportSection::SummaryPage { .. }
            | ReportSection::TableOnlyPage { .. }
            | ReportSection::CalculationPage { .. } => {}
        }
    }

    let source = build_typst_document(document);
    let images = svg_map
        .into_iter()
        .map(|(name, svg)| (PathBuf::from(name), Bytes::new(svg.into_bytes())))
        .collect::<HashMap<_, _>>();
    let world = TypstWorld::new(source, images)?;
    let result = typst::compile(&world);
    let compiled = result.output.map_err(|errors| {
        anyhow::anyhow!(
            "typst failed:\n{}",
            errors
                .iter()
                .map(|error| format!("{error:?}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    })?;

    typst_pdf::pdf(&compiled, &PdfOptions::default())
        .map_err(|error| anyhow::anyhow!("PDF failed: {error:?}"))
}

pub fn write_pdf(path: &Path, pdf_bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(path, pdf_bytes).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::render_pdf;
    use crate::report_types::{ChartRef, ReportDocument, ReportProjectMeta, ReportSection};
    use std::collections::HashMap;

    fn sample_document() -> ReportDocument {
        ReportDocument {
            project: ReportProjectMeta {
                project_name: "Proof Tower".to_string(),
                project_number: "P-001".to_string(),
                reference: "CLI-PROOF".to_string(),
                engineer: "Tester".to_string(),
                checker: "Reviewer".to_string(),
                date: "2026-04-06".to_string(),
                subject: "CLI proof report".to_string(),
                scale: "NTS".to_string(),
                revision: "0".to_string(),
                sheet_prefix: "SK".to_string(),
            },
            branch: "main".to_string(),
            version_id: "v1".to_string(),
            overall_status: "pass".to_string(),
            check_count: 1,
            pass_count: 1,
            fail_count: 0,
            sections: vec![ReportSection::SingleChartPage {
                title: "Rendered proof chart".to_string(),
                chart: ChartRef {
                    logical_name: "images/sample.svg".to_string(),
                    caption: "Rendered proof chart".to_string(),
                },
            }],
        }
    }

    #[test]
    fn render_pdf_returns_pdf_bytes() {
        let mut svgs = HashMap::new();
        svgs.insert(
            "images/sample.svg".to_string(),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"120\"><rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/><text x=\"20\" y=\"60\">proof</text></svg>".to_string(),
        );

        let pdf = render_pdf(&sample_document(), svgs).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn render_pdf_errors_when_image_missing() {
        let err = render_pdf(&sample_document(), HashMap::new()).unwrap_err();
        assert!(err.to_string().contains("Missing SVG asset"));
    }
}
