use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_pdf::PdfOptions;

use crate::data::{ReportData, ReportProjectMeta};
use crate::pdf::template::build_typst_document;
use crate::theme::PageTheme;

struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
    data: ReportData,
}

impl TypstWorld {
    fn new(content: String, data: ReportData) -> Result<Self> {
        let mut fonts = Vec::new();
        let mut book = FontBook::new();

        Self::load_bundled_fonts(&mut fonts, &mut book);
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::search_fonts(&current_dir.join("fonts"), &mut fonts, &mut book);
        if cfg!(windows) {
            Self::search_fonts(Path::new(r"C:\Windows\Fonts"), &mut fonts, &mut book);
        }

        if fonts.is_empty() {
            bail!("No readable fonts found for Typst compilation");
        }

        Ok(Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
            main: Source::new(FileId::new(None, VirtualPath::new("main.typ")), content),
            data,
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

            let Ok(buffer) = std::fs::read(font_path) else {
                continue;
            };
            let bytes = Bytes::new(buffer);
            for font in Font::iter(bytes) {
                book.push(font.info().clone());
                fonts.push(font);
            }
        }
    }

    fn load_bundled_fonts(fonts: &mut Vec<Font>, book: &mut FontBook) {
        for bytes in typst_assets::fonts() {
            let bytes = Bytes::new(bytes.to_vec());
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
        if let Some(bytes) = self.data.files.get(path) {
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

pub fn render_pdf(
    calc: &ext_calc::output::CalcOutput,
    project: &ReportProjectMeta,
    svg_map: HashMap<String, String>,
    theme: &PageTheme,
) -> Result<Vec<u8>> {
    let source = build_typst_document(calc);
    let data = ReportData::from_calc(calc, project, theme, svg_map)?;

    let world = TypstWorld::new(source, data)?;
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
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    std::fs::write(path, pdf_bytes).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{A4_PORTRAIT, TABLOID_LANDSCAPE};
    use ext_calc::output::{
        CalcOutput, TorsionalDirectionOutput, TorsionalOutput, TorsionalRow,
    };
    use std::path::PathBuf;

    fn fixture_calc_output() -> CalcOutput {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic/calc_output.json");
        let text = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn dummy_svg_map() -> HashMap<String, String> {
        let mut svgs = HashMap::new();
        for key in [
            "images/modal.svg",
            "images/base_reactions.svg",
            "images/story_force_vx.svg",
            "images/story_force_vy.svg",
            "images/story_force_my.svg",
            "images/story_force_mx.svg",
            "images/drift_wind_x.svg",
            "images/drift_wind_y.svg",
            "images/drift_seismic_x.svg",
            "images/drift_seismic_y.svg",
            "images/displacement_wind_x.svg",
            "images/displacement_wind_y.svg",
            "images/pier_shear_stress_wind.svg",
            "images/pier_shear_stress_seismic.svg",
            "images/pier_axial_gravity.svg",
            "images/pier_axial_wind.svg",
            "images/pier_axial_seismic.svg",
        ] {
            svgs.insert(key.to_string(), "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"120\"><rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/><text x=\"20\" y=\"60\">proof</text></svg>".to_string());
        }
        svgs
    }

    fn sample_torsional_row(story: &str, case: &str, ratio: f64) -> TorsionalRow {
        TorsionalRow {
            story: story.to_string(),
            case: case.to_string(),
            joint_a: "J1".to_string(),
            joint_b: "J2".to_string(),
            drift_a_steps: vec![],
            drift_b_steps: vec![],
            delta_max_steps: vec![],
            delta_avg_steps: vec![],
            ratio,
            ax: 1.2,
            ecc_ft: 0.9,
            rho: 1.0,
            is_type_a: ratio >= 1.2,
            is_type_b: ratio >= 1.4,
        }
    }

    fn build_torsional_direction(rows: Vec<TorsionalRow>) -> TorsionalDirectionOutput {
        let governing_story = rows
            .first()
            .map(|row| row.story.clone())
            .unwrap_or_else(|| "None".to_string());
        let governing_case = rows
            .first()
            .map(|row| row.case.clone())
            .unwrap_or_else(|| "None".to_string());
        let max_ratio = rows.iter().map(|row| row.ratio).fold(0.0, f64::max);
        let has_type_a = rows.iter().any(|row| row.is_type_a);
        let has_type_b = rows.iter().any(|row| row.is_type_b);
        TorsionalDirectionOutput {
            rows,
            governing_story,
            governing_case,
            governing_joints: vec!["J1".to_string(), "J2".to_string()],
            max_ratio,
            has_type_a,
            has_type_b,
        }
    }

    #[test]
    fn render_pdf_returns_pdf_bytes() {
        let calc = fixture_calc_output();
        let project = ReportProjectMeta {
            project_name: "Proof Tower".to_string(),
            subject: "CLI proof report".to_string(),
            ..ReportProjectMeta::default()
        };
        let svgs = dummy_svg_map();

        let pdf = render_pdf(&calc, &project, svgs, &TABLOID_LANDSCAPE).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn render_pdf_a4_theme_produces_pdf_bytes() {
        let calc = fixture_calc_output();
        let project = ReportProjectMeta {
            project_name: "Proof Tower".to_string(),
            subject: "A4 executive report".to_string(),
            ..ReportProjectMeta::default()
        };
        let pdf = render_pdf(&calc, &project, dummy_svg_map(), &A4_PORTRAIT).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn render_pdf_torsional_only_x_rows_produces_pdf() {
        let mut calc = fixture_calc_output();
        calc.torsional = Some(TorsionalOutput {
            x: build_torsional_direction(vec![sample_torsional_row("L10", "EQX", 1.05)]),
            y: build_torsional_direction(vec![]),
            pass: true,
        });
        let project = ReportProjectMeta::default();
        let pdf = render_pdf(&calc, &project, dummy_svg_map(), &TABLOID_LANDSCAPE).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn render_pdf_torsional_only_y_rows_produces_pdf() {
        let mut calc = fixture_calc_output();
        calc.torsional = Some(TorsionalOutput {
            x: build_torsional_direction(vec![]),
            y: build_torsional_direction(vec![sample_torsional_row("L10", "EQY", 1.11)]),
            pass: true,
        });
        let project = ReportProjectMeta::default();
        let pdf = render_pdf(&calc, &project, dummy_svg_map(), &TABLOID_LANDSCAPE).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn render_pdf_errors_when_image_missing() {
        let calc = fixture_calc_output();
        let project = ReportProjectMeta::default();
        let err = render_pdf(&calc, &project, HashMap::new(), &TABLOID_LANDSCAPE).unwrap_err();
        // Since the Typst template eagerly loads images using image("...") within the figure macros,
        // missing SVGs result in a typst compilation error (mapped through anyhow).
        assert!(err.to_string().contains("typst failed"));
    }
}
