use serde::Serialize;

use ext_calc::output::{
    BaseReactionsOutput, CalcOutput, DisplacementOutput, DriftOutput, PierAxialStressOutput,
    PierShearStressOutput, TorsionalOutput,
};

use super::format::{fmt_float, fmt_percent, fmt_with_unit};

// ── Summary ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct SummaryReportData {
    pub(super) overall_status: String,
    pub(super) check_count: u32,
    pub(super) pass_count: u32,
    pub(super) fail_count: u32,
    pub(super) branch: String,
    pub(super) version_id: String,
    pub(super) code: String,
    pub(super) lines: Vec<SummaryLineReport>,
    pub(super) checker_rows: Vec<SummaryCheckerRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct SummaryLineReport {
    pub(super) key: String,
    pub(super) status: String,
    pub(super) message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct SummaryCheckerRow {
    pub(super) check: String,
    pub(super) status: String,
    pub(super) governing_case: String,
    pub(super) governing_story: String,
    pub(super) demand: String,
    pub(super) limit: String,
    pub(super) utilization: String,
    pub(super) margin: String,
    pub(super) reason: String,
    pub(super) ratio_color_value: Option<f64>,
    pub(super) ratio_color_scale_kind: Option<String>,
}

pub(super) fn build_summary(calc: &CalcOutput) -> SummaryReportData {
    SummaryReportData {
        overall_status: calc.summary.overall_status.clone(),
        check_count: calc.summary.check_count,
        pass_count: calc.summary.pass_count,
        fail_count: calc.summary.fail_count,
        branch: calc.meta.branch.clone(),
        version_id: calc.meta.version_id.clone(),
        code: calc.meta.code.clone(),
        lines: calc
            .summary
            .lines
            .iter()
            .map(|l| SummaryLineReport {
                key: l.key.clone(),
                status: l.status.clone(),
                message: l.message.clone(),
            })
            .collect(),
        checker_rows: build_checker_rows(calc),
    }
}

pub(super) fn build_checker_rows(calc: &CalcOutput) -> Vec<SummaryCheckerRow> {
    let mut rows = Vec::new();

    if let Some(value) = &calc.drift_wind {
        rows.push(build_drift_checker_row("Drift (Wind)", &value.x, &value.y));
    }
    if let Some(value) = &calc.drift_seismic {
        rows.push(build_drift_checker_row(
            "Drift (Seismic)",
            &value.x,
            &value.y,
        ));
    }
    if let Some(value) = &calc.displacement_wind {
        rows.push(build_displacement_checker_row(
            "Displacement (Wind)",
            &value.x,
            &value.y,
        ));
    }
    if let Some(value) = &calc.torsional {
        rows.push(build_torsional_checker_row("Torsional", value));
    }
    if let Some(value) = &calc.base_reactions {
        rows.push(build_base_reactions_checker_row("Base Reactions", value));
    }
    if let Some(value) = &calc.pier_shear_stress_wind {
        rows.push(build_pier_shear_checker_row("Pier Shear (Wind)", value));
    }
    if let Some(value) = &calc.pier_shear_stress_seismic {
        rows.push(build_pier_shear_checker_row("Pier Shear (Seismic)", value));
    }
    if let Some(value) = &calc.pier_axial_stress {
        rows.push(build_pier_axial_checker_row("Pier Axial Screening", value));
    }

    rows
}

pub(super) fn build_drift_checker_row(
    check: &str,
    x: &DriftOutput,
    y: &DriftOutput,
) -> SummaryCheckerRow {
    let (governing, direction) = if x.governing.dcr >= y.governing.dcr {
        (&x.governing, "X")
    } else {
        (&y.governing, "Y")
    };
    let limit = if x.allowable_ratio > 0.0 {
        x.allowable_ratio
    } else {
        y.allowable_ratio
    };
    let utilization = governing.dcr;
    let margin = 1.0 - utilization;
    let status = if x.pass && y.pass { "pass" } else { "fail" };
    SummaryCheckerRow {
        check: check.to_string(),
        status: status.to_string(),
        governing_case: governing.output_case.clone(),
        governing_story: governing.story.clone(),
        demand: fmt_float(governing.drift_ratio),
        limit: fmt_float(limit),
        utilization: fmt_percent(utilization),
        margin: fmt_percent(margin),
        reason: format!(
            "Direction {direction}, drift ratio demand/allowable check ({:.5} / {:.5})",
            governing.drift_ratio, limit
        ),
        ratio_color_value: Some(utilization),
        ratio_color_scale_kind: Some("ratio_0_1".to_string()),
    }
}

pub(super) fn build_displacement_checker_row(
    check: &str,
    x: &DisplacementOutput,
    y: &DisplacementOutput,
) -> SummaryCheckerRow {
    let (governing, selected, direction) = if x.governing.dcr >= y.governing.dcr {
        (&x.governing, x, "X")
    } else {
        (&y.governing, y, "Y")
    };
    let limit_ft = selected
        .story_limits
        .iter()
        .find(|row| row.story == governing.story)
        .map(|row| row.limit_ft)
        .unwrap_or(selected.disp_limit.value);
    let utilization = governing.dcr;
    let margin = 1.0 - utilization;
    let status = if x.pass && y.pass { "pass" } else { "fail" };
    SummaryCheckerRow {
        check: check.to_string(),
        status: status.to_string(),
        governing_case: governing.output_case.clone(),
        governing_story: governing.story.clone(),
        demand: fmt_with_unit(governing.displacement.value * 12.0, "in"),
        limit: fmt_with_unit(limit_ft * 12.0, "in"),
        utilization: fmt_percent(utilization),
        margin: fmt_percent(margin),
        reason: format!(
            "Direction {direction}, per-level displacement limit from elevation/limit ratio"
        ),
        ratio_color_value: Some(utilization),
        ratio_color_scale_kind: Some("ratio_0_1".to_string()),
    }
}

pub(super) fn build_torsional_checker_row(
    check: &str,
    torsional: &TorsionalOutput,
) -> SummaryCheckerRow {
    let x_ratio = torsional.x.max_ratio;
    let y_ratio = torsional.y.max_ratio;
    let (direction, governing) = if x_ratio >= y_ratio {
        ("X", &torsional.x)
    } else {
        ("Y", &torsional.y)
    };
    let threshold = 1.4;
    let utilization = if threshold > 0.0 {
        governing.max_ratio / threshold
    } else {
        0.0
    };
    let margin = threshold - governing.max_ratio;
    let classification = if governing.has_type_b {
        "Type B"
    } else if governing.has_type_a {
        "Type A"
    } else {
        "None"
    };
    SummaryCheckerRow {
        check: check.to_string(),
        status: if torsional.pass { "pass" } else { "fail" }.to_string(),
        governing_case: governing.governing_case.clone(),
        governing_story: governing.governing_story.clone(),
        demand: fmt_float(governing.max_ratio),
        limit: fmt_float(threshold),
        utilization: fmt_percent(utilization),
        margin: fmt_float(margin),
        reason: format!(
            "Direction {direction}, ratio basis dmax/davg, classification {classification}"
        ),
        ratio_color_value: Some(governing.max_ratio),
        ratio_color_scale_kind: Some("torsion_thresholds_1_2_1_4".to_string()),
    }
}

pub(super) fn build_base_reactions_checker_row(
    check: &str,
    base: &BaseReactionsOutput,
) -> SummaryCheckerRow {
    let min_ratio = base.direction_x.ratio.min(base.direction_y.ratio);
    let utilization = min_ratio;
    let margin = min_ratio - 1.0;
    let pass = base.direction_x.pass && base.direction_y.pass;
    SummaryCheckerRow {
        check: check.to_string(),
        status: if pass { "pass" } else { "fail" }.to_string(),
        governing_case: format!(
            "X {} / Y {}",
            base.direction_x.rsa_case, base.direction_y.rsa_case
        ),
        governing_story: "-".to_string(),
        demand: fmt_float(min_ratio),
        limit: ">= 1.000".to_string(),
        utilization: fmt_percent(utilization),
        margin: fmt_float(margin),
        reason: format!(
            "RSA/ELF ratio check, X={:.5}, Y={:.5}",
            base.direction_x.ratio, base.direction_y.ratio
        ),
        ratio_color_value: None,
        ratio_color_scale_kind: None,
    }
}

pub(super) fn build_pier_shear_checker_row(
    check: &str,
    value: &PierShearStressOutput,
) -> SummaryCheckerRow {
    if !value.supported {
        return SummaryCheckerRow {
            check: check.to_string(),
            status: "warn".to_string(),
            governing_case: "-".to_string(),
            governing_story: "-".to_string(),
            demand: "-".to_string(),
            limit: "-".to_string(),
            utilization: "-".to_string(),
            margin: "-".to_string(),
            reason: value
                .support_note
                .clone()
                .unwrap_or_else(|| "Check is not available for configured code".to_string()),
            ratio_color_value: None,
            ratio_color_scale_kind: None,
        };
    }

    let util_individual = if value.limit_individual > 0.0 {
        value.max_individual_ratio / value.limit_individual
    } else {
        0.0
    };
    let util_average = if value.limit_average > 0.0 {
        value.max_average_ratio / value.limit_average
    } else {
        0.0
    };
    let utilization = util_individual.max(util_average);
    let margin = 1.0 - utilization;
    SummaryCheckerRow {
        check: check.to_string(),
        status: if value.pass { "pass" } else { "fail" }.to_string(),
        governing_case: "-".to_string(),
        governing_story: "-".to_string(),
        demand: format!(
            "ind {:.3}, avg {:.3}",
            value.max_individual_ratio, value.max_average_ratio
        ),
        limit: format!(
            "ind {:.3}, avg {:.3}",
            value.limit_individual, value.limit_average
        ),
        utilization: fmt_percent(utilization),
        margin: fmt_percent(margin),
        reason: "Stress-ratio checks against individual and average limits".to_string(),
        ratio_color_value: Some(utilization),
        ratio_color_scale_kind: Some("ratio_0_1".to_string()),
    }
}

pub(super) fn build_pier_axial_checker_row(
    check: &str,
    value: &PierAxialStressOutput,
) -> SummaryCheckerRow {
    SummaryCheckerRow {
        check: check.to_string(),
        status: if value.pass { "pass" } else { "fail" }.to_string(),
        governing_case: value.governing.combo.clone(),
        governing_story: value.governing.story.clone(),
        demand: format!(
            "{} (signed {})",
            fmt_with_unit(value.governing.fa.value, "ksi"),
            fmt_with_unit(value.governing.fa_signed.value, "ksi")
        ),
        limit: fmt_with_unit(0.85 * value.governing.fc_ksi, "ksi"),
        utilization: fmt_percent(value.governing.dcr),
        margin: fmt_percent(1.0 - value.governing.dcr),
        reason: "Preliminary axial screening check".to_string(),
        ratio_color_value: Some(value.governing.dcr),
        ratio_color_scale_kind: Some("ratio_0_1".to_string()),
    }
}
