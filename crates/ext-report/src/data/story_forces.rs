use serde::Serialize;

use ext_calc::output::StoryForcesOutput;

// ── Story Forces ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct StoryForcesReportData {
    pub(super) rows: Vec<StoryForcesReportRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct StoryForcesReportRow {
    pub(super) story: String,
    pub(super) max_vx_kip: f64,
    pub(super) max_my_kip_ft: f64,
    pub(super) max_vy_kip: f64,
    pub(super) max_mx_kip_ft: f64,
}

pub(super) fn build_story_forces(story_forces: &StoryForcesOutput) -> StoryForcesReportData {
    StoryForcesReportData {
        rows: story_forces
            .rows
            .iter()
            .map(|row| StoryForcesReportRow {
                story: row.story.clone(),
                max_vx_kip: row.max_vx_kip,
                max_my_kip_ft: row.max_my_kip_ft,
                max_vy_kip: row.max_vy_kip,
                max_mx_kip_ft: row.max_mx_kip_ft,
            })
            .collect(),
    }
}
