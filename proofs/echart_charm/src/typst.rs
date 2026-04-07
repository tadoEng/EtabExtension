// FILE: D:\repo\echart_charm\src\typst.rs
// Replace the entire file with this content.

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_pdf::PdfOptions;

// ─── Image layout system ──────────────────────────────────────────────────────
//
// Page geometry (tabloid landscape 17×11in):
//   Top margin    0.40in  — breathing room above content rect
//   Bottom margin 0.65in  — reserved for 0.60in title-block + 0.05in bleed
//   Left/right    0.50in each
//
// Content rect = width: 100%, height: 100%  (fills the margin box exactly).
// Usable inner area (inside 2pt stroke + 20pt inset):
//   Width  ≈ 15.55in
//   Height ≈ 9.47in
//
// Image height budget per layout:
//   Single           → 7.2in  (centered)
//   SideBySide       → 6.8in  (left / right, 1fr each)
//   Stacked          → 3.5in  each (top / bottom)
//   Three            → 3.6in  for all three
//   TableAndImage    → 6.2in  (right column)
//   TableOnly        → no images
//   ChartThenTable   → chart page: 7.2in full; table page: full-width table
//
// Rule: ALWAYS constrain images by `height:`, never `width: 100%`.
// SVG intrinsic ratio 800×550 (≈ 1.45) → width:100% = 15.6in → 10.8in → overflow.

#[derive(Debug, Clone)]
pub enum ImageLayout {
    /// One image, centered, full height budget
    Single,
    /// Two images side by side, equal width
    SideBySide,
    /// Two images stacked vertically
    Stacked,
    /// Three images: two side-by-side on top, one centered below
    Three,
    /// One data table (inline Typst markup) on the left, one image on the right
    TableAndImage,
    /// One data table spanning full width (no images on this page)
    TableOnly,
    /// Chart on page A (full width), full-width table on page B.
    /// Use when the table is too tall to share a page with the chart.
    /// Automatically consumes two sheet numbers.
    ChartThenTable,
}

// ─── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub project_name: String,
    pub project_num:  String,
    pub reference:    String,
    pub engineer:     String,
    pub checker:      String,
    pub date:         String,
    pub subject:      String,
    pub scale:        String,
    pub sheet:        String,   // base sheet, e.g. "SK-01"
    pub revision:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundationElement {
    pub id:            String,
    pub demand:        f64,
    pub demand_unit:   String,
    pub capacity:      f64,
    pub capacity_unit: String,
    pub dcr:           f64,
    pub format:        String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationStep {
    pub description: String,
    pub formula:     String,   // Typst math markup
    pub result:      String,   // Typst math markup, or empty
    pub note:        String,   // plain text note below result, or empty
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationSection {
    pub title: String,
    pub steps: Vec<CalculationStep>,
}

/// One named image to embed. The logical filename must match the key
/// used in the SVG map passed to generate_report().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub logical_name: String,   // e.g. "images/story_shear.svg"
    pub caption:      String,
}

/// A page of image/table content with an explicit layout.
#[derive(Debug, Clone)]
pub struct ImagePage {
    pub heading: String,
    pub layout:  ImageLayout,
    pub images:  Vec<ImageRef>,      // must match layout arity
    /// Inline Typst markup for TableAndImage / TableOnly / ChartThenTable.
    pub table_markup: Option<String>,
    /// Heading override for the table page when using ChartThenTable.
    /// Falls back to `heading` when None.
    pub table_heading: Option<String>,
}

/// Top-level report descriptor.
#[derive(Debug, Clone)]
pub struct ReportData {
    pub project: ProjectData,
    pub pages:   Vec<ReportPage>,
}

/// Each logical page in the report body (after cover).
/// Note: ChartThenTable emits two physical pages.
#[derive(Debug, Clone)]
pub enum ReportPage {
    Images(ImagePage),
    Calculations(Vec<CalculationSection>),
    DcrTable(Vec<FoundationElement>),
}

// ─── TypstWorld ───────────────────────────────────────────────────────────────

struct TypstWorld {
    library:     LazyHash<Library>,
    book:        LazyHash<FontBook>,
    fonts:       Vec<Font>,
    main:        Source,
    image_cache: HashMap<PathBuf, Bytes>,
}

impl TypstWorld {
    fn new(content: String, extra_images: HashMap<PathBuf, Bytes>) -> Self {
        let mut fonts = Vec::new();
        let mut book  = FontBook::new();

        println!("loading fonts...");
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::search_fonts(&current_dir.join("fonts"), &mut fonts, &mut book);
        Self::search_fonts(Path::new(r"C:\Windows\Fonts"), &mut fonts, &mut book);

        if fonts.is_empty() { panic!("no fonts could be loaded"); }
        println!("fonts loaded: {}", fonts.len());

        let main = Source::new(
            FileId::new(None, VirtualPath::new("main.typ")),
            content,
        );

        // In-memory SVG bytes take priority; disk images fill the rest.
        let mut image_cache = extra_images;
        let images_dir = current_dir.join("images");
        if let Ok(entries) = fs::read_dir(&images_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "svg") {
                        if let Ok(data) = fs::read(&path) {
                            let key = path.strip_prefix(&current_dir).unwrap_or(&path).to_path_buf();
                            image_cache.entry(key).or_insert_with(|| Bytes::new(data));
                        }
                    }
                }
            }
        }

        Self {
            library: LazyHash::new(Library::default()),
            book:    LazyHash::new(book),
            fonts, main, image_cache,
        }
    }

    fn search_fonts(path: &Path, fonts: &mut Vec<Font>, book: &mut FontBook) {
        if !path.exists() { return; }
        for entry in walkdir::WalkDir::new(path)
            .follow_links(true)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.is_file() {
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                if matches!(ext.as_str(), "ttf" | "otf" | "ttc" | "otc") {
                    if let Ok(data) = fs::read(p) {
                        let buf = Bytes::new(data);
                        for font in Font::iter(buf) {
                            book.push(font.info().clone());
                            fonts.push(font);
                        }
                    }
                }
            }
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library>  { &self.library }
    fn book(&self)    -> &LazyHash<FontBook> { &self.book    }
    fn main(&self)    -> FileId              { self.main.id() }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() { Ok(self.main.clone()) }
        else { Err(typst::diag::FileError::NotFound(id.vpath().as_rootless_path().into())) }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.vpath().as_rootless_path();
        if let Some(b) = self.image_cache.get(Path::new(path))                     { return Ok(b.clone()); }
        if let Some(b) = self.image_cache.get(&PathBuf::from("images").join(path)) { return Ok(b.clone()); }
        Err(typst::diag::FileError::NotFound(path.into()))
    }

    fn font(&self, index: usize) -> Option<Font>        { self.fonts.get(index).cloned() }
    fn today(&self, _: Option<i64>) -> Option<Datetime> { None }
}

// ─── PDF compilation ─────────────────────────────────────────────────────────

fn compile_to_pdf(content: &str, extra_images: HashMap<PathBuf, Bytes>) -> Result<Vec<u8>> {
    let world  = TypstWorld::new(content.to_string(), extra_images);
    let result = typst::compile(&world);
    let doc    = result.output.map_err(|errs| {
        anyhow::anyhow!("typst failed:\n{}", errs.iter().map(|e| format!("{e:?}")).collect::<Vec<_>>().join("\n"))
    })?;
    typst_pdf::pdf(&doc, &PdfOptions::default())
        .map_err(|e| anyhow::anyhow!("PDF failed: {e:?}"))
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Primary entry point. Pass all SVG strings keyed by their logical name.
/// The report page order is determined by `data.pages`.
pub fn generate_report(
    output_path: &str,
    data:        &ReportData,
    svgs:        HashMap<String, String>,
) -> Result<()> {
    let extra_images: HashMap<PathBuf, Bytes> = svgs
        .into_iter()
        .map(|(k, v)| (PathBuf::from(k), Bytes::new(v.into_bytes())))
        .collect();

    let content = generate_typst(data);
    let pdf     = compile_to_pdf(&content, extra_images)?;
    fs::write(output_path, pdf)?;
    println!("report saved: {output_path}");
    Ok(())
}

// ─── Typst source generation ──────────────────────────────────────────────────

fn generate_typst(data: &ReportData) -> String {
    let p = &data.project;
    let base_sheet: u32 = p.sheet.trim_start_matches("SK-").parse().unwrap_or(1);
    let mut sheet_counter = base_sheet;
    let mut out = String::new();

    out.push_str(&global_setup());
    out.push_str(&title_block_fn());
    out.push_str(&helper_fns());

    // PAGE 1: Cover
    let s = format!("SK-{:02}", sheet_counter);
    out.push_str(&cover_page(&p.project_name, &p.subject, &tb_call(p, &s)));
    sheet_counter += 1;

    // Body pages
    for page in &data.pages {
        out.push_str("\n#pagebreak()\n");
        let s = format!("SK-{:02}", sheet_counter);

        match page {
            ReportPage::Images(img_page) => {
                let rendered = image_page(img_page, &tb_call(p, &s));

                if rendered.contains("__CHART_THEN_TABLE_SPLIT__") {
                    // ChartThenTable: split into two physical pages.
                    let mut parts = rendered.splitn(2, "__CHART_THEN_TABLE_SPLIT__");
                    let chart_part = parts.next().unwrap_or("");
                    let table_part = parts.next().unwrap_or("");

                    out.push_str(chart_part);

                    // Second page — advance counter, emit table page.
                    sheet_counter += 1;
                    let s2 = format!("SK-{:02}", sheet_counter);
                    out.push_str("\n#pagebreak()\n");
                    out.push_str(&format!(
                        "#content_rect[\n  {}\n]\n{}",
                        table_part,
                        tb_call(p, &s2),
                    ));
                } else {
                    out.push_str(&rendered);
                }
            }

            ReportPage::Calculations(sections) => {
                out.push_str(&calc_page(sections, &tb_call(p, &s)));
            }

            ReportPage::DcrTable(elements) => {
                for (i, chunk) in elements.chunks(22).enumerate() {
                    if i > 0 {
                        out.push_str("\n#pagebreak()\n");
                        sheet_counter += 1;
                    }
                    let s2 = format!("SK-{:02}", sheet_counter);
                    out.push_str(&dcr_page(chunk, i == 0, &tb_call(p, &s2)));
                }
            }
        }

        sheet_counter += 1;
    }

    out
}

// ─── Global Typst setup ───────────────────────────────────────────────────────

fn global_setup() -> String {
    // Page geometry (tabloid landscape 17×11in):
    //   Top margin    0.40in — breathing room above content rect
    //   Bottom margin 0.65in — reserved for the 0.60in title-block + 0.05in bleed
    //   Left/right    0.50in each
    //
    // content_rect uses height: 100% which fills exactly the margin box.
    // title_block is placed at dy: 0pt from the bottom+left anchor, so it sits
    // immediately below (outside) the content rect with zero gap.
    r##"
#set text(font: "Arial", size: 9pt)
#set page(
  width: 17in,
  height: 11in,
  margin: (top: 0.40in, left: 0.50in, right: 0.50in, bottom: 0.65in),
)
"##.into()
}

// ─── Helper Typst functions ───────────────────────────────────────────────────

fn helper_fns() -> String {
    r##"
#let content_rect(body) = rect(
  width: 100%,
  height: 100%,
  stroke: 2pt + black,
  inset: 20pt,
  body,
)

#let dcr_color(v) = {
  if v >= 1.0       { rgb("#CC0000") }
  else if v >= 0.95 { rgb("#E06000") }
  else if v >= 0.85 { rgb("#B08000") }
  else              { rgb("#1A7A1A") }
}

"##.into()
}

// ─── Title block function ─────────────────────────────────────────────────────
//
// Total width: 16in across 6 columns:
//   Logo 1.5in | Project 3.8in | Drawing title 4.6in |
//   Ref/Rev 1.6in | Drawn/Checked/Date 1.9in | Scale/Sheet 2.6in
//
// All columns share a single 0.60in row height.
// Internal column dividers: 0.5pt; outer border: 1.5pt.
// dy: 0pt — block sits exactly at the bottom margin edge, flush below content_rect.

fn title_block_fn() -> String {
    r##"
#let title_block(project, proj_num, reference, engineer, checker, date, subject, scale, sheet, revision) = {
  // dy: 0pt — anchors at the page bottom-margin edge.
  // The content_rect above fills height: 100% so they meet with zero gap.
  place(bottom + left, dy: 0pt)[
    #set text(font: "Arial")
    #block(width: 16in, stroke: (thickness: 1.5pt, paint: black))[
      #grid(
        columns: (1.5in, 3.8in, 4.6in, 1.6in, 1.9in, 2.6in),
        rows: (0.6in),
        stroke: (x: (thickness: 0.5pt, paint: black), y: none),

        align(center + horizon)[
          #stack(spacing: 0pt,
            text(size: 12pt, weight: "bold", fill: rgb("#E63D1F"))[Thornton],
            text(size: 12pt, weight: "bold", fill: rgb("#003A70"))[Tomasetti],
          )
        ],

        pad(x: 6pt, y: 4pt)[
          #text(size: 5.5pt, fill: luma(110))[PROJECT] \
          #text(size: 8pt, weight: "bold")[#project] \
          #v(2pt)
          #text(size: 5.5pt, fill: luma(110))[PROJECT NO.] \
          #text(size: 7.5pt)[#proj_num]
        ],

        pad(x: 6pt, y: 4pt)[
          #text(size: 5.5pt, fill: luma(110))[DRAWING TITLE] \
          #text(size: 9pt, weight: "bold")[#subject]
        ],

        grid(
          columns: (1fr), rows: (0.3in, 0.3in),
          stroke: (x: none, y: (thickness: 0.5pt, paint: black)),
          pad(x: 5pt, y: 3pt)[
            #text(size: 5.5pt, fill: luma(110))[REFERENCE] \
            #text(size: 7.5pt)[#reference]
          ],
          pad(x: 5pt, y: 3pt)[
            #text(size: 5.5pt, fill: luma(110))[REVISION] \
            #text(size: 8pt, weight: "bold")[#revision]
          ],
        ),

        grid(
          columns: (1fr), rows: (0.3in, 0.3in),
          stroke: (x: none, y: (thickness: 0.5pt, paint: black)),
          pad(x: 5pt, y: 3pt)[
            #text(size: 5.5pt, fill: luma(110))[DRAWN BY]
            #h(1fr)
            #text(size: 5.5pt, fill: luma(110))[CHECKED BY] \
            #text(size: 8.5pt, weight: "bold")[#engineer]
            #h(1fr)
            #text(size: 8.5pt, weight: "bold")[#checker]
          ],
          pad(x: 5pt, y: 3pt)[
            #text(size: 5.5pt, fill: luma(110))[DATE] \
            #text(size: 7.5pt)[#date]
          ],
        ),

        grid(
          columns: (1fr), rows: (0.3in, 0.3in),
          stroke: (x: none, y: (thickness: 0.5pt, paint: black)),
          pad(x: 5pt, y: 3pt)[
            #text(size: 5.5pt, fill: luma(110))[SCALE] \
            #text(size: 8pt)[#scale]
          ],
          align(center + horizon)[
            #text(size: 5.5pt, fill: luma(110))[SHEET] \
            #text(size: 17pt, weight: "bold")[#sheet]
          ],
        ),
      )
    ]
  ]
}

"##.into()
}

/// Emit a `#title_block(...)` call with all project fields and the given sheet.
fn tb_call(p: &ProjectData, sheet: &str) -> String {
    format!(
        r##"#title_block("{}", "{}", "{}", "{}", "{}", "{}", "{}", "{}", "{}", "{}")"##,
        p.project_name, p.project_num, p.reference,
        p.engineer, p.checker, p.date,
        p.subject, p.scale, sheet, p.revision,
    )
}

// ─── Page generators ──────────────────────────────────────────────────────────

fn cover_page(project_name: &str, subject: &str, tb: &str) -> String {
    format!(r##"
#content_rect[
  #align(center + horizon)[
    #text(size: 44pt, weight: "bold", fill: rgb("#003A70"))[{project_name}]
    #v(14pt)
    #text(size: 20pt, fill: luma(80))[Foundation Design Report]
    #v(8pt)
    #text(size: 12pt, fill: luma(120))[{subject}]
  ]
]
{tb}
"##)
}

// ── Image page ────────────────────────────────────────────────────────────────

fn image_page(pg: &ImagePage, tb: &str) -> String {
    let heading = format!(
        r##"#align(center)[
  #text(size: 14pt, weight: "bold", fill: rgb("#003A70"))[{}]
]
#v(10pt)"##,
        pg.heading.to_uppercase()
    );

    let body = match pg.layout {
        ImageLayout::Single => {
            let img = &pg.images[0];
            format!(r##"
#align(center)[
  #figure(
    image("{}", height: 7.2in),
    caption: [{}],
    supplement: none,
  )
]"##, img.logical_name, img.caption)
        }

        ImageLayout::SideBySide => {
            let (a, b) = (&pg.images[0], &pg.images[1]);
            format!(r##"
#grid(
  columns: (1fr, 1fr),
  gutter: 14pt,
  align(center)[
    #figure(image("{}", height: 6.8in), caption: [{}], supplement: none)
  ],
  align(center)[
    #figure(image("{}", height: 6.8in), caption: [{}], supplement: none)
  ],
)"##, a.logical_name, a.caption, b.logical_name, b.caption)
        }

        ImageLayout::Stacked => {
            let (a, b) = (&pg.images[0], &pg.images[1]);
            format!(r##"
#align(center)[
  #figure(image("{}", height: 3.5in), caption: [{}], supplement: none)
]
#v(8pt)
#align(center)[
  #figure(image("{}", height: 3.5in), caption: [{}], supplement: none)
]"##, a.logical_name, a.caption, b.logical_name, b.caption)
        }

        ImageLayout::Three => {
            let (a, b, c) = (&pg.images[0], &pg.images[1], &pg.images[2]);
            format!(r##"
#grid(
  columns: (1fr, 1fr),
  gutter: 12pt,
  align(center)[
    #figure(image("{}", height: 3.6in), caption: [{}], supplement: none)
  ],
  align(center)[
    #figure(image("{}", height: 3.6in), caption: [{}], supplement: none)
  ],
)
#v(6pt)
#align(center)[
  #figure(image("{}", height: 3.6in), caption: [{}], supplement: none)
]"##,
                a.logical_name, a.caption,
                b.logical_name, b.caption,
                c.logical_name, c.caption)
        }

        ImageLayout::TableAndImage => {
            let img   = &pg.images[0];
            let table = pg.table_markup.as_deref().unwrap_or("// no table markup");
            format!(r##"
#grid(
  columns: (1fr, 1fr),
  gutter: 16pt,
  align(top)[
    {}
  ],
  align(center)[
    #figure(image("{}", height: 6.2in), caption: [{}], supplement: none)
  ],
)"##, table, img.logical_name, img.caption)
        }

        ImageLayout::TableOnly => {
            let table = pg.table_markup.as_deref().unwrap_or("// no table markup");
            format!("\n{}\n", table)
        }

        // Chart body only — the second page is emitted below via sentinel split.
        ImageLayout::ChartThenTable => {
            let img = &pg.images[0];
            format!(r##"
#align(center)[
  #figure(
    image("{}", height: 7.2in),
    caption: [{}],
    supplement: none,
  )
]"##, img.logical_name, img.caption)
        }
    };

    // ── ChartThenTable: emit chart page + sentinel + table page body ──────────
    if matches!(pg.layout, ImageLayout::ChartThenTable) {
        let table_heading_str = pg.table_heading.as_deref().unwrap_or(&pg.heading);
        let table_heading_markup = format!(
            r##"#align(center)[
  #text(size: 14pt, weight: "bold", fill: rgb("#003A70"))[{}]
]
#v(10pt)"##,
            table_heading_str.to_uppercase()
        );
        let table_body = pg.table_markup.as_deref().unwrap_or("// no table markup");

        let chart_page = format!(r##"
#content_rect[
  {heading}
  {body}
]
{tb}"##);

        // The sentinel is stripped by generate_typst; the table part is wrapped
        // in content_rect with the next sheet's tb_call by the caller.
        return format!("{chart_page}\n__CHART_THEN_TABLE_SPLIT__\n{table_heading_markup}\n{table_body}\n");
    }

    format!(r##"
#content_rect[
  {}
  {}
]
{}"##, heading, body, tb)
}

// ── Calculation page ──────────────────────────────────────────────────────────

fn calc_page(sections: &[CalculationSection], tb: &str) -> String {
    let mut body = String::from(r##"
#align(center)[
  #text(size: 14pt, weight: "bold", fill: rgb("#003A70"))[DESIGN CALCULATIONS]
]
#v(14pt)
"##);

    for section in sections {
        body.push_str(&format!(r##"
#rect(
  width: 100%,
  fill: rgb("#EEF3F8"),
  stroke: (left: 3pt + rgb("#003A70"), rest: none),
  inset: (x: 10pt, y: 5pt),
)[
  #text(size: 10pt, weight: "bold", fill: rgb("#003A70"))[{}]
]
#v(6pt)
"##, section.title));

        let col_break   = (section.steps.len() + 1) / 2;
        let left_steps  = &section.steps[..col_break];
        let right_steps = &section.steps[col_break..];

        body.push_str(r##"#grid(columns: (1fr, 1fr), gutter: 20pt, align(top)["##);
        body.push_str(&render_steps(left_steps));
        body.push_str(r##"], align(top)["##);
        body.push_str(&render_steps(right_steps));
        body.push_str("#])\n#v(10pt)\n");
    }

    format!(r##"
#content_rect[
  {}
]
{}"##, body, tb)
}

fn render_steps(steps: &[CalculationStep]) -> String {
    let mut out = String::new();
    for (i, step) in steps.iter().enumerate() {
        out.push_str(&format!(r##"
#text(size: 8pt, weight: "bold")[{}. {}]
#v(2pt)
$ {} $
"##, i + 1, step.description, step.formula));

        if !step.result.is_empty() {
            out.push_str(&format!(
                r##"#text(size: 8pt, fill: rgb("#003A70"), weight: "bold")[→ $ {} $]
"##, step.result));
        }
        if !step.note.is_empty() {
            out.push_str(&format!(
                r##"#text(size: 7pt, fill: luma(120))[{}]
"##, step.note));
        }
        out.push_str("#v(8pt)\n");
    }
    out
}

// ── DCR table page ────────────────────────────────────────────────────────────

fn dcr_page(elements: &[FoundationElement], is_first: bool, tb: &str) -> String {
    let heading = if is_first {
        r##"#align(center)[
  #text(size: 14pt, weight: "bold", fill: rgb("#003A70"))[DEMAND-CAPACITY RATIOS]
]
#v(10pt)"##
    } else {
        r##"#align(center)[
  #text(size: 12pt, weight: "bold", fill: rgb("#003A70"))[DEMAND-CAPACITY RATIOS (CONTINUED)]
]
#v(8pt)"##
    };

    let mut table = r##"#table(
  columns: (2fr, 1.5fr, 1.5fr, 0.8fr, 0.8fr, 1.2fr),
  stroke: 0.5pt + luma(180),
  fill: (col, row) => {
    if row == 0 { rgb("#003A70") }
    else if calc.odd(row) { rgb("#EEF3F8") }
    else { white }
  },
  align: (col, row) => {
    if row == 0 { center + horizon }
    else if col > 2 { center + horizon }
    else { left + horizon }
  },
  inset: (x: 7pt, y: 5pt),
  text(fill: white, weight: "bold", size: 8pt)[Element ID],
  text(fill: white, weight: "bold", size: 8pt)[Demand],
  text(fill: white, weight: "bold", size: 8pt)[Capacity],
  text(fill: white, weight: "bold", size: 8pt)[DCR],
  text(fill: white, weight: "bold", size: 8pt)[Status],
  text(fill: white, weight: "bold", size: 8pt)[Format],
"##.to_string();

    for elem in elements {
        let status = if elem.dcr >= 1.0 { "FAIL" } else if elem.dcr >= 0.95 { "Check" } else { "OK" };
        table.push_str(&format!(
            "  [{}], [{:.1} {}], [{:.1} {}], \
             [#text(fill: dcr_color({:.2}), weight: \"bold\")[{:.2}]], \
             [#text(fill: dcr_color({:.2}))[{}]], [{}],\n",
            elem.id,
            elem.demand, elem.demand_unit,
            elem.capacity, elem.capacity_unit,
            elem.dcr, elem.dcr,
            elem.dcr, status,
            elem.format,
        ));
    }
    table.push_str(")\n");

    let legend = if is_first {
        r##"
#v(8pt)
#text(size: 7pt, fill: luma(100))[
  *Color code:*
  #box(fill: rgb("#1A7A1A"), width: 7pt, height: 7pt, radius: 1pt) #h(2pt) ≤ 0.85 (OK)
  #h(10pt)
  #box(fill: rgb("#B08000"), width: 7pt, height: 7pt, radius: 1pt) #h(2pt) 0.85–0.94
  #h(10pt)
  #box(fill: rgb("#E06000"), width: 7pt, height: 7pt, radius: 1pt) #h(2pt) 0.95–0.99
  #h(10pt)
  #box(fill: rgb("#CC0000"), width: 7pt, height: 7pt, radius: 1pt) #h(2pt) ≥ 1.0 (FAIL)
]"##
    } else { "" };

    format!(r##"
#content_rect[
  {heading}
  {table}{legend}
]
{tb}"##)
}

// ─── Sample data builders ─────────────────────────────────────────────────────

/// Build a complete example ReportData with all page types, including
/// a ChartThenTable demo page.
pub fn example_report_data(project: ProjectData) -> ReportData {
    ReportData {
        project,
        pages: vec![
            // Page 2: Two charts side by side
            ReportPage::Images(ImagePage {
                heading:       "Structural Analysis — Load Distribution".into(),
                layout:        ImageLayout::SideBySide,
                images:        vec![
                    ImageRef { logical_name: "images/base_reactions.svg".into(), caption: "Base Reactions by Load Case".into() },
                    ImageRef { logical_name: "images/force_disp.svg".into(),     caption: "Force vs Displacement".into() },
                ],
                table_markup:  None,
                table_heading: None,
            }),

            // Page 3: Single story shear chart
            ReportPage::Images(ImagePage {
                heading:       "Lateral Load Analysis".into(),
                layout:        ImageLayout::Single,
                images:        vec![
                    ImageRef { logical_name: "images/story_shear.svg".into(), caption: "Story Shear — X and Y Directions".into() },
                ],
                table_markup:  None,
                table_heading: None,
            }),

            // Page 4a/4b: Drift chart (page 4) then full drift table (page 5).
            // Use this pattern whenever the table is too tall to share a page.
            ReportPage::Images(ImagePage {
                heading:       "Wind Drift Review".into(),
                layout:        ImageLayout::ChartThenTable,
                images:        vec![
                    ImageRef { logical_name: "images/drift_wind.svg".into(), caption: "Maximum drift ratio per story under wind loading.".into() },
                ],
                table_markup:  Some(load_summary_table()),  // replace with real drift table markup
                table_heading: Some("Wind Drift — Story Data".into()),
            }),

            // Hand calculations
            ReportPage::Calculations(example_calculations()),

            // DCR table (auto-paginates at 22 rows)
            ReportPage::DcrTable(generate_random_elements(30)),
        ],
    }
}

/// Inline Typst markup for a load summary table (used by TableAndImage layout).
pub fn load_summary_table() -> String {
    r##"
#text(size: 9pt, weight: "bold", fill: rgb("#003A70"))[Load Summary]
#v(6pt)
#table(
  columns: (2fr, 1fr, 1fr, 1fr),
  stroke: 0.5pt + luma(180),
  fill: (col, row) => if row == 0 { rgb("#003A70") } else if calc.odd(row) { rgb("#EEF3F8") } else { white },
  align: (col, row) => if row == 0 { center } else if col == 0 { left } else { center },
  inset: (x: 6pt, y: 5pt),
  text(fill: white, weight: "bold", size: 8pt)[Load Case],
  text(fill: white, weight: "bold", size: 8pt)[Fx (kips)],
  text(fill: white, weight: "bold", size: 8pt)[Fy (kips)],
  text(fill: white, weight: "bold", size: 8pt)[Fz (kips)],
  [Dead (D)],        [0.0],  [0.0],  [2450.0],
  [Live (L)],        [0.0],  [0.0],   [820.0],
  [Super. DL (SDL)], [0.0],  [0.0],   [310.0],
  [Wind X (Wx)],   [185.0],  [0.0],    [12.0],
  [Wind Y (Wy)],     [0.0],[172.0],     [8.0],
  [Seismic X (Ex)],[240.0],  [0.0],    [18.0],
  [Seismic Y (Ey)],  [0.0],[228.0],    [15.0],
  [1.2D+1.6L],       [0.0],  [0.0], [4252.0],
  [1.2D+1.0Ex+0.3Ey],[72.0],[228.0],[2988.0],
)
"##.into()
}

/// Sample hand-calculation sections for structural engineering.
pub fn example_calculations() -> Vec<CalculationSection> {
    vec![
        CalculationSection {
            title: "Dead Load — Slab Self Weight".into(),
            steps: vec![
                CalculationStep {
                    description: "Concrete unit weight".into(),
                    formula:     r#"gamma_c = 150 "pcf""#.into(),
                    result:      "".into(),
                    note:        "Normal weight concrete per ACI 318-14 §19.2.3".into(),
                },
                CalculationStep {
                    description: "Slab thickness".into(),
                    formula:     r#"h_s = 8 "in" = 0.667 "ft""#.into(),
                    result:      "".into(),
                    note:        "".into(),
                },
                CalculationStep {
                    description: "Slab dead load".into(),
                    formula:     r#"w_"DL" = gamma_c times h_s"#.into(),
                    result:      r#"w_"DL" = 150 times 0.667 = 100 "psf""#.into(),
                    note:        "".into(),
                },
                CalculationStep {
                    description: "Superimposed dead load (MEP + finishes)".into(),
                    formula:     r#"w_"SDL" = 25 "psf""#.into(),
                    result:      "".into(),
                    note:        "Per architectural finish schedule".into(),
                },
            ],
        },
        CalculationSection {
            title: "Live Load".into(),
            steps: vec![
                CalculationStep {
                    description: "Office occupancy (ASCE 7-22 Table 4.3-1)".into(),
                    formula:     r#"L_o = 50 "psf""#.into(),
                    result:      "".into(),
                    note:        "".into(),
                },
                CalculationStep {
                    description: "Tributary area".into(),
                    formula:     r#"A_T = 28 "ft" times 30 "ft" = 840 "ft"^2"#.into(),
                    result:      "".into(),
                    note:        "".into(),
                },
                CalculationStep {
                    description: "Live load reduction (ASCE 7-22 §4.7)".into(),
                    formula:     r#"L = L_o (0.25 + 15 / sqrt(K_"LL" A_T))"#.into(),
                    result:      r#"L = 50 (0.25 + 15 / sqrt(2 times 840)) = 38.9 "psf""#.into(),
                    note:        "K_LL = 2 for two-way slabs".into(),
                },
            ],
        },
        CalculationSection {
            title: "Factored Load (LRFD)".into(),
            steps: vec![
                CalculationStep {
                    description: "Governing combo: 1.2D + 1.6L (ASCE 7-22 §2.3.1)".into(),
                    formula:     r#"w_u = 1.2 w_"DL" + 1.2 w_"SDL" + 1.6 L"#.into(),
                    result:      r#"w_u = 1.2(100) + 1.2(25) + 1.6(38.9) = 212.2 "psf""#.into(),
                    note:        "".into(),
                },
                CalculationStep {
                    description: "Factored column load".into(),
                    formula:     r#"P_u = w_u times A_T"#.into(),
                    result:      r#"P_u = 212.2 times 840 / 1000 = 178.3 "kips""#.into(),
                    note:        "".into(),
                },
            ],
        },
        CalculationSection {
            title: "Lateral — Story Shear".into(),
            steps: vec![
                CalculationStep {
                    description: "Base shear (ASCE 7-22 §12.8)".into(),
                    formula:     r#"V = C_s W"#.into(),
                    result:      r#"V = 0.094 times 3580 = 336.5 "kips""#.into(),
                    note:        "Cs = SDS / (R/Ie) = 0.9 / (6/1.0) = 0.150 → 0.094 governs".into(),
                },
                CalculationStep {
                    description: "Story force distribution (§12.8.3)".into(),
                    formula:     r#"F_x = C_"vx" V, quad C_"vx" = (w_x h_x^k) / (sum w_i h_i^k)"#.into(),
                    result:      "".into(),
                    note:        "k = 1.33 (interpolated, T = 0.66 s)".into(),
                },
            ],
        },
    ]
}

// ─── Random test data ─────────────────────────────────────────────────────────

pub fn generate_random_elements(count: usize) -> Vec<FoundationElement> {
    let mut rng = rand::thread_rng();
    let types = [
        ("FTG",  "kips", "kips", "Standard"),
        ("COL",  "kips", "kips", "HSS12x12"),
        ("BEAM", "k-ft", "k-ft", "W18x50"),
        ("SLAB", "psf",  "psf",  "8in"),
        ("WALL", "plf",  "plf",  "12in CMU"),
    ];
    (1..=count).map(|i| {
        let (pre, du, cu, fmt) = types[rng.gen_range(0..types.len())];
        let cap = rng.gen_range(100.0_f64..1000.0);
        let dcr = rng.gen_range(0.45_f64..0.98);
        let dem = (cap * dcr * 10.0).round() / 10.0;
        FoundationElement {
            id:            format!("{pre}-{i:03}"),
            demand:        dem,
            demand_unit:   du.into(),
            capacity:      (cap * 10.0).round() / 10.0,
            capacity_unit: cu.into(),
            dcr:           (dcr * 100.0).round() / 100.0,
            format:        fmt.into(),
        }
    }).collect()
}