use serde::Serialize;

use ext_calc::output::PierAxialStressOutput;

// ── Pier Axial (minimal — for assumptions page) ──────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PierAxialReportData {
    pub(super) phi_axial: f64,
    pub(super) pass: bool,
}

pub(super) fn build_pier_axial(axial: &PierAxialStressOutput) -> PierAxialReportData {
    PierAxialReportData {
        phi_axial: axial.phi_axial,
        pass: axial.pass,
    }
}
