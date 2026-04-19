mod pages;
mod partials;
mod pier_shear_pages;
mod review_pages;
mod sequence;
mod standard_pages;

use crate::pdf::procedures;
use ext_calc::output::CalcOutput;

pub fn build_typst_document(calc: &CalcOutput) -> String {
    let mut doc = String::new();

    partials::append_all(&mut doc);
    standard_pages::append(&mut doc);
    pier_shear_pages::append(&mut doc);
    review_pages::append(&mut doc);
    procedures::append_definitions(&mut doc);
    sequence::append(&mut doc, calc);

    doc
}

pub(crate) use partials::write_all_to_dir as write_typst_partials_to_dir;

#[cfg(test)]
mod tests {
    use super::{
        build_typst_document,
        pages::{PageId, PageLayout, build_report_pages},
    };
    use crate::data::{ReportData, ReportProjectMeta};
    use crate::theme::TABLOID_LANDSCAPE;
    use ext_calc::CalcRunner;
    use ext_calc::code_params::CodeParams;
    use ext_calc::output::CalcOutput;
    use ext_db::config::Config;
    use std::collections::{HashMap, HashSet};
    use std::path::Path;
    use std::path::PathBuf;

    fn fixture_calc_output() -> CalcOutput {
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic");
        let path = fixture_dir.join("calc_output.json");
        if path.exists() {
            let text = std::fs::read_to_string(path).expect("fixture json should be readable");
            serde_json::from_str(&text).expect("fixture json should deserialize")
        } else {
            let config = Config::load(&fixture_dir).expect("fixture config should load");
            let params = CodeParams::from_config(&config).expect("code params should build");
            CalcRunner::run_all(
                fixture_dir.as_path(),
                fixture_dir.as_path(),
                &params,
                "fixture",
                "main",
            )
            .expect("fixture calc output should build")
        }
    }

    fn typst_block<'a>(typst: &'a str, start: &str, end: &str) -> &'a str {
        let start_idx = typst
            .find(start)
            .unwrap_or_else(|| panic!("missing Typst block start: {start}"));
        let rest = &typst[start_idx..];
        let end_idx = rest
            .find(end)
            .unwrap_or_else(|| panic!("missing Typst block end after {start}: {end}"));
        &rest[..end_idx]
    }

    fn without_optional_pages() -> CalcOutput {
        let mut calc = fixture_calc_output();
        calc.modal = None;
        calc.base_reactions = None;
        calc.story_forces = None;
        calc.drift_wind = None;
        calc.drift_seismic = None;
        calc.displacement_wind = None;
        calc.torsional = None;
        calc.pier_shear_stress_wind = None;
        calc.pier_shear_stress_seismic = None;
        calc.pier_axial_stress = None;
        calc
    }

    fn dummy_svg_map() -> HashMap<String, String> {
        [
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
            "images/torsional_x.svg",
            "images/torsional_y.svg",
            "images/pier_shear_stress_wind_x.svg",
            "images/pier_shear_stress_wind_y.svg",
            "images/pier_shear_stress_wind_avg.svg",
            "images/pier_shear_stress_seismic_x.svg",
            "images/pier_shear_stress_seismic_y.svg",
            "images/pier_shear_stress_seismic_avg.svg",
            "images/pier_axial_gravity.svg",
            "images/pier_axial_wind.svg",
            "images/pier_axial_seismic.svg",
        ]
        .into_iter()
        .map(|key| {
            (
                key.to_string(),
                "<svg xmlns=\"http://www.w3.org/2000/svg\"/>".to_string(),
            )
        })
        .collect()
    }

    #[test]
    fn page_registry_keeps_core_pages_and_current_order() {
        let calc = fixture_calc_output();
        let ids = build_report_pages(&calc)
            .iter()
            .map(|page| page.id)
            .collect::<Vec<_>>();

        assert_eq!(ids.first(), Some(&PageId::Cover));
        assert_eq!(ids.get(1), Some(&PageId::Summary));
        assert_eq!(ids.get(2), Some(&PageId::ScopeLimitations));
        assert!(ids.contains(&PageId::Modal));
        assert!(ids.contains(&PageId::PierAxialSeismic));
        assert_eq!(ids.last(), Some(&PageId::CalculationTrace));

        let unique = ids.iter().copied().collect::<HashSet<_>>();
        assert_eq!(unique.len(), ids.len(), "page ids should be unique");
    }

    #[test]
    fn page_registry_omits_optional_pages_when_data_is_absent() {
        let calc = without_optional_pages();
        let pages = build_report_pages(&calc);
        let ids = pages.iter().map(|page| page.id).collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                PageId::Cover,
                PageId::Summary,
                PageId::ScopeLimitations,
                PageId::CalculationTrace,
            ]
        );
        assert!(
            pages
                .iter()
                .all(|page| page.layout != PageLayout::TwoCharts)
        );
    }

    #[test]
    fn page_registry_dependencies_are_available_as_virtual_files() {
        let calc = fixture_calc_output();
        let report_data = ReportData::from_calc(
            &calc,
            &ReportProjectMeta::default(),
            &TABLOID_LANDSCAPE,
            dummy_svg_map(),
        )
        .unwrap();

        for page in build_report_pages(&calc) {
            for path in page.data_files.iter().chain(page.image_files.iter()) {
                assert!(
                    report_data.files.contains_key(Path::new(path)),
                    "missing dependency for {:?}: {path}",
                    page.id
                );
            }
        }
    }

    #[test]
    fn directional_sections_render_review_page_before_verification_page() {
        let calc = fixture_calc_output();
        let typst = build_typst_document(&calc);

        for (review, verify) in [
            (
                "#torsion-review-pair-page([Torsional Irregularity Review]",
                "#torsion-verify-pair-page([Torsional Irregularity Verification]",
            ),
            (
                "#pier-shear-review-pair-page([Pier Shear Wind Review]",
                "#pier-shear-verify-pair-page([Pier Shear Wind Verification]",
            ),
            (
                "#pier-shear-review-pair-page([Pier Shear Seismic Review]",
                "#pier-shear-verify-pair-page([Pier Shear Seismic Verification]",
            ),
            (
                "#pier-shear-average-review-page([Pier Shear Wind Average Review]",
                "#pier-shear-average-verify-page([Pier Shear Wind Average Verification]",
            ),
            (
                "#pier-shear-average-review-page([Pier Shear Seismic Average Review]",
                "#pier-shear-average-verify-page([Pier Shear Seismic Average Verification]",
            ),
        ] {
            let review_idx = typst
                .find(review)
                .unwrap_or_else(|| panic!("missing review marker: {review}"));
            let verify_idx = typst
                .find(verify)
                .unwrap_or_else(|| panic!("missing verification marker: {verify}"));
            assert!(
                review_idx < verify_idx,
                "review marker should appear before verification marker for '{review}'"
            );
        }
    }

    #[test]
    fn template_uses_directional_pier_shear_assets_and_removes_axial_screening_page() {
        let calc = fixture_calc_output();
        let typst = build_typst_document(&calc);

        for image in [
            "images/pier_shear_stress_wind_x.svg",
            "images/pier_shear_stress_wind_y.svg",
            "images/pier_shear_stress_seismic_x.svg",
            "images/pier_shear_stress_seismic_y.svg",
        ] {
            assert!(
                typst.contains(image),
                "missing directional image asset {image}"
            );
        }

        assert!(
            !typst.contains("#pier-axial-assumptions(\"pier_axial_stress.json\")"),
            "axial preliminary-screening page call should be removed from sequence"
        );

        assert!(
            typst.contains("#show figure: set block(breakable: false)"),
            "figures should be non-breakable to avoid orphaned chart/table fragments"
        );
        assert!(
            typst.contains("#set figure(numbering: \"1\", outlined: false)"),
            "figures should be numbered globally"
        );
        assert!(
            !typst.contains("numbering: none"),
            "old unnumbered figure setting should be removed"
        );
        assert!(
            typst.contains("#let ext-figure(path, caption-text, height)"),
            "chart images should route through the shared ext-figure helper"
        );
        assert!(
            typst.contains("#let two-chart-row(chart1, cap1, chart2, cap2)"),
            "side-by-side chart review pages should share two-chart-row"
        );
        assert!(
            typst.contains("governing-summary("),
            "review pages should use structured governing-summary callouts"
        );
        assert!(
            typst
                .matches("text(size: parse-pt(theme.title-size)")
                .count()
                == 1,
            "page-title should be the only title-size text definition"
        );
        assert!(
            typst
                .matches("text(size: parse-pt(theme.label-size)")
                .count()
                == 6,
            "label-size text calls should be limited to helper definitions and worked-example result helpers"
        );
        assert!(
            !typst.contains("text(size: parse-pt(theme.body-size)"),
            "body-note/section-label helpers should replace all inline body-size text calls"
        );
        assert!(
            !typst.contains("text(size: title-size") && !typst.contains("text(size: label-size"),
            "typography migration should not use local size aliases"
        );
        for helper_usage in ["page-title[", "section-label[", "body-note[", "ref-note["] {
            assert!(
                typst.contains(helper_usage),
                "expected direct typography helper usage: {helper_usage}"
            );
        }
        assert!(
            typst.contains("block(breakable: false)[#stack(spacing: 4pt, section-label[X Direction], drift-table(data.x))]"),
            "drift verification columns should be unbreakable and use typography helpers"
        );
        assert!(
            typst.contains("block(breakable: false)[#stack(spacing: 4pt, section-label[X Direction], displacement-table(data.x))]"),
            "displacement verification columns should be unbreakable and use typography helpers"
        );
        assert!(
            typst.matches("with-divider(").count() >= 5,
            "with-divider helper should be used in verification tables and worked examples"
        );
        for (start, end) in [
            (
                "#let two-chart-row(chart1, cap1, chart2, cap2)",
                "#let with-divider(left-content, right-content)",
            ),
            (
                "#let story-force-review-page(title, chart1, cap1, chart2, cap2)",
                "#let drift-review-pair-page",
            ),
            ("#let drift-review-pair-page", "#let drift-verify-pair-page"),
            (
                "#let displacement-review-pair-page",
                "#let displacement-verify-pair-page",
            ),
            (
                "#let torsion-review-pair-page",
                "#let torsion-verify-pair-page",
            ),
            (
                "#let pier-shear-review-pair-page",
                "#let pier-shear-verify-pair-page",
            ),
        ] {
            assert!(
                !typst_block(&typst, start, end).contains("with-divider("),
                "chart review helper should not use with-divider: {start}"
            );
        }
        assert!(
            !typst.contains("#drift-verify-pair-page([Wind Drift Verification]")
                && !typst.contains("#drift-verify-pair-page([Seismic Drift Verification]")
                && !typst
                    .contains("#displacement-verify-pair-page([Wind Displacement Verification]")
                && !typst.contains("#drift-verify-pair-page(")
                && !typst.contains("#displacement-verify-pair-page("),
            "duplicative drift/displacement verification page calls should be removed"
        );
        let table_header_count = typst.matches("table.header(").count();
        let inline_repeat_header_count = typst.matches("table.header(repeat: true,").count();
        assert_eq!(
            table_header_count,
            inline_repeat_header_count + 1,
            "every ordinary table.header should use repeat: true; only repeating-header may call table.header with multiline args"
        );
        assert!(
            typst.contains("table.header(\n    repeat: true,")
                && !typst.contains("table.header(repeat: false"),
            "repeating-header should call table.header(repeat: true) and no header should disable repeats"
        );
        assert!(
            typst.matches("repeating-header(").count() >= 4,
            "repeating-header should be defined and used for modal, torsion, and pier-shear directional tables"
        );
        assert!(
            typst.contains(
                "block(breakable: false)[\n      #align(center)[\n        #ext-figure(chart-file"
            ),
            "pier-shear-average-review-page chart and summary should be wrapped in a non-breakable block"
        );
        assert!(
            typst.contains("parse-pt(theme.label-size) + 2pt"),
            "worked-example boxed results should use direct label-size + 2pt sizing"
        );
        assert!(
            typst.contains("enum("),
            "scope-limitations page should use a Typst enum list"
        );
        assert!(
            !typst.contains("align(center)[image(chart1"),
            "story-force chart1 must not contain unevaluated image() call"
        );
        assert!(
            !typst.contains("align(center)[image(chart2"),
            "story-force chart2 must not contain unevaluated image() call"
        );
        assert!(
            !typst.contains("align(center)[\n      figure(\n        image(chart-file"),
            "average shear review must not contain unevaluated figure() call"
        );
        assert!(
            !typst.contains("align(center)[text(size: parse-pt(theme.caption-size)"),
            "story-force captions should not leak as raw caption text inside stacks"
        );
        assert!(
            !typst.contains("image(\"images/"),
            "literal chart image paths should be wrapped by ext-figure"
        );
        for raw_dynamic_image in [
            "image(chart1",
            "image(chart2",
            "image(chart-x",
            "image(chart-y",
            "image(chart-file",
        ] {
            assert!(
                !typst.contains(raw_dynamic_image),
                "dynamic chart image call should be wrapped by ext-figure: {raw_dynamic_image}"
            );
        }
        for caption in [
            "Story Shear Vx (kip)",
            "Story Moment My (kip·ft)",
            "Wind Drift Ratio — X Direction",
            "Seismic Drift Ratio — Y Direction",
            "Wind Displacement — X Direction (in)",
            "Torsional Ratio — X Direction",
            "Pier Shear Stress Ratio Wind — X Walls",
            "Pier Axial Stress — Seismic (ksi)",
        ] {
            assert!(
                typst.contains(caption),
                "missing expected chart caption: {caption}"
            );
        }
        assert!(
            typst.contains("#single-chart-page([Pier Axial - Gravity]")
                && typst.contains("#single-chart-page([Pier Axial - Wind]")
                && typst.contains("#single-chart-page([Pier Axial - Seismic]"),
            "pier axial should remain three separate pages"
        );
        for procedure_marker in [
            "#let given-table(pairs)",
            "#let calc-step(n, formula, substitution, result)",
            "#let calc-result(label, value, pass)",
            "if is-executive",
        ] {
            assert!(
                typst.contains(procedure_marker),
                "missing worked-example marker: {procedure_marker}"
            );
        }
        assert!(
            !typst.contains("text(size: 9pt"),
            "worked examples should not hardcode text below theme.label-size"
        );
    }

    #[test]
    fn template_source_is_assembled_from_named_partials_and_page_registry() {
        let calc = fixture_calc_output();
        let typst = build_typst_document(&calc);

        for partial in [
            "// ext-report partial: styles.typ",
            "// ext-report partial: page_setup.typ",
            "// ext-report partial: layouts.typ",
            "// ext-report partial: components.typ",
        ] {
            assert!(typst.contains(partial), "missing partial marker {partial}");
        }

        assert!(typst.contains("// ext-report page sequence: registry"));
        assert!(typst.contains("two-charts-page("));
        assert!(typst.contains("chart-table-layout("));
        assert!(typst.contains("with-divider("));
        assert!(typst.contains("#single-chart-page("));
    }
}
