use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ext_calc::output::CalcOutput;
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_pdf::PdfOptions;

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
}

#[derive(Debug, Clone)]
pub struct ReportImage {
    pub logical_name: String,
    pub caption: String,
}

#[derive(Debug, Clone)]
pub struct ReportInput {
    pub project: ReportProjectMeta,
    pub calc: CalcOutput,
    pub images: Vec<ReportImage>,
}

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

pub fn build_typst_document(input: &ReportInput) -> String {
    let mut doc = String::new();
    doc.push_str("#set text(font: \"Arial\", size: 9pt)\n");
    doc.push_str("#set page(width: 17in, height: 11in, margin: (top: 0.5in, left: 0.5in, right: 0.5in, bottom: 0.7in))\n");
    doc.push_str("#set par(justify: false)\n\n");

    doc.push_str(&format!(
        "#align(center + horizon)[\n  #text(size: 26pt, weight: \"bold\")[{}]\n  #v(10pt)\n  #text(size: 16pt)[Structural Check Report]\n  #v(6pt)\n  #text(fill: rgb(\"#555555\"))[{}]\n]\n\n",
        escape_text(&input.project.project_name),
        escape_text(&input.project.subject),
    ));

    doc.push_str("#grid(columns: (1fr, 1fr), gutter: 18pt,\n");
    doc.push_str(&format!(
        "  [*Reference:* {}\\\n*Project No.:* {}\\\n*Revision:* {}],\n",
        escape_text(&input.project.reference),
        escape_text(&input.project.project_number),
        escape_text(&input.project.revision),
    ));
    doc.push_str(&format!(
        "  [*Engineer:* {}\\\n*Checker:* {}\\\n*Date:* {}],\n)\n\n",
        escape_text(&input.project.engineer),
        escape_text(&input.project.checker),
        escape_text(&input.project.date),
    ));

    doc.push_str("== Summary\n\n");
    doc.push_str(&format!(
        "- Overall status: {}\n- Active checks: {}\n- Passed: {}\n- Failed: {}\n- Branch/version: {}/{}\n",
        escape_text(&input.calc.summary.overall_status),
        input.calc.summary.check_count,
        input.calc.summary.pass_count,
        input.calc.summary.fail_count,
        escape_text(&input.calc.meta.branch),
        escape_text(&input.calc.meta.version_id),
    ));

    if !input.calc.summary.lines.is_empty() {
        doc.push_str("\n== Check Notes\n\n");
        for line in &input.calc.summary.lines {
            doc.push_str(&format!(
                "- *{}* [{}] {}\n",
                escape_text(&line.key),
                escape_text(&line.status),
                escape_text(&line.message),
            ));
        }
    }

    for image in &input.images {
        doc.push_str("\n#pagebreak()\n");
        doc.push_str(&format!(
            "= {}\n\n#figure(\n  image(\"{}\", height: 6.6in),\n  caption: [{}],\n)\n",
            escape_text(&image.caption),
            escape_text(&image.logical_name),
            escape_text(&image.caption),
        ));
    }

    doc
}

pub fn compile_pdf(input: &ReportInput, svg_map: HashMap<String, String>) -> Result<Vec<u8>> {
    for image in &input.images {
        if !svg_map.contains_key(&image.logical_name) {
            bail!("Missing SVG asset for '{}'", image.logical_name);
        }
    }

    let source = build_typst_document(input);
    let images = svg_map
        .into_iter()
        .map(|(name, svg)| (PathBuf::from(name), Bytes::new(svg.into_bytes())))
        .collect::<HashMap<_, _>>();
    let world = TypstWorld::new(source, images)?;
    let result = typst::compile(&world);
    let document = result.output.map_err(|errors| {
        anyhow::anyhow!(
            "typst failed:\n{}",
            errors
                .iter()
                .map(|error| format!("{error:?}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    })?;

    typst_pdf::pdf(&document, &PdfOptions::default())
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

fn escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
}

#[cfg(test)]
mod tests {
    use super::{ReportImage, ReportInput, ReportProjectMeta, build_typst_document, compile_pdf};
    use ext_calc::output::CalcOutput;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn fixture_calc_output() -> CalcOutput {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic/calc_output.json");
        let text = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn sample_input() -> ReportInput {
        ReportInput {
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
            },
            calc: fixture_calc_output(),
            images: vec![ReportImage {
                logical_name: "images/sample.svg".to_string(),
                caption: "Rendered proof chart".to_string(),
            }],
        }
    }

    #[test]
    fn typst_document_uses_tabloid_landscape() {
        let source = build_typst_document(&sample_input());
        assert!(source.contains("width: 17in"));
        assert!(source.contains("height: 11in"));
    }

    #[test]
    fn compile_pdf_returns_pdf_bytes() {
        let mut svgs = HashMap::new();
        svgs.insert(
            "images/sample.svg".to_string(),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"120\"><rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/><text x=\"20\" y=\"60\">proof</text></svg>".to_string(),
        );

        let pdf = compile_pdf(&sample_input(), svgs).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn compile_pdf_errors_when_image_missing() {
        let err = compile_pdf(&sample_input(), HashMap::new()).unwrap_err();
        assert!(err.to_string().contains("Missing SVG asset"));
    }
}
