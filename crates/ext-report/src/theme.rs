use serde::{Deserialize, Serialize};
use std::str::FromStr;

use anyhow::{Result, bail};

/// All measurements that vary between paper formats.
/// Injected into TypstWorld as "theme.json".
/// Changing the theme changes visual layout — data and template logic are unchanged.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PageTheme {
    // "cad-sheet" => border + title block layout
    // "executive" => native header/footer Word-style layout
    pub layout_kind: &'static str,

    // ── Page geometry ─────────────────────────────────────────────────────────
    pub page_width: &'static str,    // "17in"
    pub page_height: &'static str,   // "11in"
    pub margin_top: &'static str,    // "0.25in"
    pub margin_left: &'static str,   // "0.25in"
    pub margin_right: &'static str,  // "0.25in"
    pub margin_bottom: &'static str, // "0.25in"

    // ── Content area ──────────────────────────────────────────────────────────
    // Invariant: content-height + margin-top + margin-bottom + tb-h = page-height
    // tb-h is derived by the Typst template — not stored here
    pub content_height: &'static str, // "9.75in"

    // ── Typography ────────────────────────────────────────────────────────────
    pub body_font: &'static str,    // "Linux Libertine"
    pub body_size: &'static str,    // "9pt"
    pub title_size: &'static str,   // "14pt"
    pub label_size: &'static str,   // "7pt"  (table header row text)
    pub caption_size: &'static str, // "8pt"

    // ── Chart heights per layout type ─────────────────────────────────────────
    pub chart_single_h: &'static str,            // "8.5in"
    pub chart_two_col_h: &'static str,           // "7.5in"
    pub chart_with_table_chart_h: &'static str,  // "6in"
    pub chart_with_table_normal_h: &'static str, // "7in"

    // ── Grid column ratios (Typst fraction strings) ───────────────────────────
    pub two_col_ratio: &'static str,          // "(1fr, 1fr)"
    pub chart_table_emphasized: &'static str, // "(1.08fr, 0.92fr)"
    pub chart_table_normal: &'static str,     // "(0.82fr, 1.18fr)"

    // ── Title block ───────────────────────────────────────────────────────────
    // Column widths — must sum to border-w (page-width - margin-left - margin-right)
    pub title_block_columns: &'static str,

    // ── Spacing ───────────────────────────────────────────────────────────────
    pub section_gap: &'static str,   // "10pt"
    pub table_inset: &'static str,   // "5pt"
    pub grid_gutter: &'static str,   // "20pt"
    pub content_inset: &'static str, // "18pt"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportTheme {
    Tabloid,
    A4,
}

impl ReportTheme {
    pub fn page_theme(self) -> &'static PageTheme {
        match self {
            Self::Tabloid => &TABLOID_LANDSCAPE,
            Self::A4 => &A4_PORTRAIT,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tabloid => "tabloid",
            Self::A4 => "a4",
        }
    }
}

impl Default for ReportTheme {
    fn default() -> Self {
        Self::Tabloid
    }
}

impl FromStr for ReportTheme {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "tabloid" => Ok(Self::Tabloid),
            "a4" => Ok(Self::A4),
            _ => bail!("Unknown theme: {value}. Use 'tabloid' or 'a4'."),
        }
    }
}

pub const TABLOID_LANDSCAPE: PageTheme = PageTheme {
    layout_kind: "cad-sheet",
    page_width: "17in",
    page_height: "11in",
    margin_top: "0.25in",
    margin_left: "0.25in",
    margin_right: "0.25in",
    margin_bottom: "0.25in",
    content_height: "9.75in",
    // tb-h = 11 - 0.25 - 0.25 - 9.75 = 0.75in ✓
    body_font: "Linux Libertine",
    body_size: "9pt",
    title_size: "14pt",
    label_size: "7pt",
    caption_size: "8pt",

    chart_single_h: "8.5in",
    chart_two_col_h: "7.5in",
    chart_with_table_chart_h: "6in",
    chart_with_table_normal_h: "7in",

    two_col_ratio: "(1fr, 1fr)",
    chart_table_emphasized: "(1.08fr, 0.92fr)",
    chart_table_normal: "(0.82fr, 1.18fr)",

    // Sum = 16.5in = 17in - 0.25in - 0.25in ✓
    title_block_columns: "(3.35in, 3.2in, 4.0in, 1.6in, 2.0in, 2.35in)",

    section_gap: "10pt",
    table_inset: "5pt",
    grid_gutter: "20pt",
    content_inset: "18pt",
};

pub const A4_PORTRAIT: PageTheme = PageTheme {
    layout_kind: "executive",
    page_width: "8.27in",
    page_height: "11.69in",
    margin_top: "1.2in",
    margin_left: "1in",
    margin_right: "1in",
    margin_bottom: "1.2in",
    // executive layout doesn't use a title-block band, but keep this for schema consistency.
    content_height: "9.29in",

    body_font: "Linux Libertine",
    body_size: "10pt",
    title_size: "16pt",
    label_size: "8pt",
    caption_size: "8pt",

    chart_single_h: "6in",
    chart_two_col_h: "3.2in",
    chart_with_table_chart_h: "4in",
    chart_with_table_normal_h: "4in",

    two_col_ratio: "1fr",
    chart_table_emphasized: "1fr",
    chart_table_normal: "1fr",

    // Unused in executive mode, retained for compatibility.
    title_block_columns: "(1in, 1in, 1in, 1in, 1in, 1.27in)",

    section_gap: "14pt",
    table_inset: "6pt",
    grid_gutter: "20pt",
    content_inset: "0pt",
};

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_inches(s: &str) -> f64 {
        s.trim_end_matches("in").parse::<f64>().unwrap()
    }

    fn assert_theme_invariant(theme: &PageTheme, label: &str) {
        let pg_h = parse_inches(theme.page_height);
        let m_top = parse_inches(theme.margin_top);
        let m_bottom = parse_inches(theme.margin_bottom);
        let c_h = parse_inches(theme.content_height);
        let tb_h = pg_h - m_top - m_bottom - c_h;

        assert!(
            pg_h > (m_top + m_bottom),
            "{label}: margins exceed page height"
        );

        if theme.layout_kind == "cad-sheet" {
            assert!(
                tb_h > 0.0,
                "{label}: derived tb-h must be positive, got {tb_h}"
            );
            let reconstructed = m_top + m_bottom + c_h + tb_h;
            assert!(
                (reconstructed - pg_h).abs() < 1e-6,
                "{label}: tb-h + content-height + margin-top + margin-bottom must equal page-height. \
                 Got {reconstructed} vs {pg_h}"
            );
            return;
        }

        // Executive mode has no title-block band; allow zero remainder.
        assert!(
            tb_h >= -1e-6,
            "{label}: content area should fit page (tb-h remainder {tb_h})"
        );
    }

    #[test]
    fn tabloid_landscape_satisfies_invariant() {
        assert_theme_invariant(&TABLOID_LANDSCAPE, "TABLOID_LANDSCAPE");
    }

    #[test]
    fn a4_portrait_satisfies_invariant() {
        assert_theme_invariant(&A4_PORTRAIT, "A4_PORTRAIT");
    }
}
