use ext_calc::output::CalcOutput;

use crate::pdf::procedures;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PageId {
    Cover,
    Summary,
    ScopeLimitations,
    Modal,
    BaseReactions,
    StoryForcesX,
    StoryForcesY,
    DriftWindReview,
    DriftWindTables,
    DriftSeismicReview,
    DriftSeismicTables,
    DisplacementWindReview,
    DisplacementWindTables,
    TorsionalReview,
    TorsionalTables,
    PierShearWindReview,
    PierShearWindTables,
    PierShearWindAverageReview,
    PierShearSeismicReview,
    PierShearSeismicTables,
    PierShearSeismicAverageReview,
    PierAxialGravity,
    PierAxialWind,
    PierAxialSeismic,
    VerificationExamples,
}

impl PageId {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Summary => "summary",
            Self::ScopeLimitations => "scope-limitations",
            Self::Modal => "modal",
            Self::BaseReactions => "base-reactions",
            Self::StoryForcesX => "story-forces-x",
            Self::StoryForcesY => "story-forces-y",
            Self::DriftWindReview => "drift-wind-review",
            Self::DriftWindTables => "drift-wind-tables",
            Self::DriftSeismicReview => "drift-seismic-review",
            Self::DriftSeismicTables => "drift-seismic-tables",
            Self::DisplacementWindReview => "displacement-wind-review",
            Self::DisplacementWindTables => "displacement-wind-tables",
            Self::TorsionalReview => "torsional-review",
            Self::TorsionalTables => "torsional-tables",
            Self::PierShearWindReview => "pier-shear-wind-review",
            Self::PierShearWindTables => "pier-shear-wind-tables",
            Self::PierShearWindAverageReview => "pier-shear-wind-average-review",
            Self::PierShearSeismicReview => "pier-shear-seismic-review",
            Self::PierShearSeismicTables => "pier-shear-seismic-tables",
            Self::PierShearSeismicAverageReview => "pier-shear-seismic-average-review",
            Self::PierAxialGravity => "pier-axial-gravity",
            Self::PierAxialWind => "pier-axial-wind",
            Self::PierAxialSeismic => "pier-axial-seismic",
            Self::VerificationExamples => "verification-examples",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageLayout {
    Cover,
    Summary,
    Limitations,
    OneChart,
    TwoCharts,
    TwoTables,
    ChartTable,
    Procedure,
}

impl PageLayout {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Summary => "summary",
            Self::Limitations => "limitations",
            Self::OneChart => "one-chart",
            Self::TwoCharts => "two-charts",
            Self::TwoTables => "two-tables",
            Self::ChartTable => "chart-table",
            Self::Procedure => "procedure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageAvailability {
    Always,
    WhenCalcDataPresent(&'static str),
    WhenProcedurePageEnabled,
}

impl PageAvailability {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::WhenCalcDataPresent(field) => field,
            Self::WhenProcedurePageEnabled => "procedure-page-enabled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TypstCall {
    source: &'static str,
}

impl TypstCall {
    const fn new(source: &'static str) -> Self {
        Self { source }
    }

    pub(super) fn source(self) -> &'static str {
        self.source
    }
}

#[derive(Debug, Clone)]
pub(super) struct ReportPage {
    pub(super) id: PageId,
    pub(super) heading: &'static str,
    pub(super) layout: PageLayout,
    pub(super) availability: PageAvailability,
    pub(super) data_files: &'static [&'static str],
    pub(super) image_files: &'static [&'static str],
    pub(super) typst_call: TypstCall,
}

impl ReportPage {
    const fn new(
        id: PageId,
        heading: &'static str,
        layout: PageLayout,
        availability: PageAvailability,
        data_files: &'static [&'static str],
        image_files: &'static [&'static str],
        typst_call: &'static str,
    ) -> Self {
        Self {
            id,
            heading,
            layout,
            availability,
            data_files,
            image_files,
            typst_call: TypstCall::new(typst_call),
        }
    }
}

pub(super) fn build_report_pages(calc: &CalcOutput) -> Vec<ReportPage> {
    let mut pages = vec![
        ReportPage::new(
            PageId::Cover,
            "Cover",
            PageLayout::Cover,
            PageAvailability::Always,
            &["summary.json"],
            &[],
            "#cover-page()",
        ),
        ReportPage::new(
            PageId::Summary,
            "Report Summary",
            PageLayout::Summary,
            PageAvailability::Always,
            &["summary.json"],
            &[],
            "#summary-page()",
        ),
        ReportPage::new(
            PageId::ScopeLimitations,
            "Scope and Limitations",
            PageLayout::Limitations,
            PageAvailability::Always,
            &[],
            &[],
            "#scope-limitations-page()",
        ),
    ];

    if calc.modal.is_some() {
        pages.push(ReportPage::new(
            PageId::Modal,
            "Modal Participation",
            PageLayout::TwoTables,
            PageAvailability::WhenCalcDataPresent("modal"),
            &["modal.json"],
            &[],
            "#modal-page()",
        ));
    }

    if calc.base_reactions.is_some() {
        pages.push(ReportPage::new(
            PageId::BaseReactions,
            "Base Reaction Review",
            PageLayout::ChartTable,
            PageAvailability::WhenCalcDataPresent("base_reactions"),
            &["base_reactions.json"],
            &["images/base_reactions.svg"],
            "#base-reactions-page()",
        ));
    }

    if calc.story_forces.is_some() {
        pages.push(ReportPage::new(
            PageId::StoryForcesX,
            "Story Forces - X Direction",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("story_forces"),
            &[],
            &["images/story_force_vx.svg", "images/story_force_my.svg"],
            "#story-force-review-page([Story Forces — X Direction], \"images/story_force_vx.svg\", [Story Shear Vx (kip)], \"images/story_force_my.svg\", [Story Moment My (kip·ft)])",
        ));
        pages.push(ReportPage::new(
            PageId::StoryForcesY,
            "Story Forces - Y Direction",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("story_forces"),
            &[],
            &["images/story_force_vy.svg", "images/story_force_mx.svg"],
            "#story-force-review-page([Story Forces — Y Direction], \"images/story_force_vy.svg\", [Story Shear Vy (kip)], \"images/story_force_mx.svg\", [Story Moment Mx (kip·ft)])",
        ));
    }

    if calc.drift_wind.is_some() {
        pages.push(ReportPage::new(
            PageId::DriftWindReview,
            "Wind Drift Review",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("drift_wind"),
            &["drift_wind.json"],
            &["images/drift_wind_x.svg", "images/drift_wind_y.svg"],
            "#let dw = json(\"drift_wind.json\")\n#drift-review-pair-page([Wind Drift Review], dw, \"images/drift_wind_x.svg\", [Wind Drift Ratio — X Direction], \"images/drift_wind_y.svg\", [Wind Drift Ratio — Y Direction])",
        ));
        pages.push(ReportPage::new(
            PageId::DriftWindTables,
            "Wind Drift Tables",
            PageLayout::TwoTables,
            PageAvailability::WhenCalcDataPresent("drift_wind"),
            &["drift_wind.json"],
            &[],
            "#let dw = json(\"drift_wind.json\")\n#drift-tables-pair-page([Wind Drift Tables], dw)",
        ));
    }

    if calc.drift_seismic.is_some() {
        pages.push(ReportPage::new(
            PageId::DriftSeismicReview,
            "Seismic Drift Review",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("drift_seismic"),
            &["drift_seismic.json"],
            &["images/drift_seismic_x.svg", "images/drift_seismic_y.svg"],
            "#let ds = json(\"drift_seismic.json\")\n#drift-review-pair-page([Seismic Drift Review], ds, \"images/drift_seismic_x.svg\", [Seismic Drift Ratio — X Direction], \"images/drift_seismic_y.svg\", [Seismic Drift Ratio — Y Direction])",
        ));
        pages.push(ReportPage::new(
            PageId::DriftSeismicTables,
            "Seismic Drift Tables",
            PageLayout::TwoTables,
            PageAvailability::WhenCalcDataPresent("drift_seismic"),
            &["drift_seismic.json"],
            &[],
            "#let ds = json(\"drift_seismic.json\")\n#drift-tables-pair-page([Seismic Drift Tables], ds)",
        ));
    }

    if calc.displacement_wind.is_some() {
        pages.push(ReportPage::new(
            PageId::DisplacementWindReview,
            "Wind Displacement Review",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("displacement_wind"),
            &["displacement_wind.json"],
            &["images/displacement_wind_x.svg", "images/displacement_wind_y.svg"],
            "#let dpw = json(\"displacement_wind.json\")\n#displacement-review-pair-page([Wind Displacement Review], dpw, \"images/displacement_wind_x.svg\", [Wind Displacement — X Direction (in)], \"images/displacement_wind_y.svg\", [Wind Displacement — Y Direction (in)])",
        ));
        pages.push(ReportPage::new(
            PageId::DisplacementWindTables,
            "Wind Displacement Tables",
            PageLayout::TwoTables,
            PageAvailability::WhenCalcDataPresent("displacement_wind"),
            &["displacement_wind.json"],
            &[],
            "#let dpw = json(\"displacement_wind.json\")\n#displacement-tables-pair-page([Wind Displacement Tables], dpw)",
        ));
    }

    if calc.torsional.is_some() {
        pages.push(ReportPage::new(
            PageId::TorsionalReview,
            "Torsional Irregularity Review",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("torsional"),
            &["torsional.json"],
            &["images/torsional_x.svg", "images/torsional_y.svg"],
            "#let tor = json(\"torsional.json\")\n#torsion-review-pair-page([Torsional Irregularity Review], tor, \"images/torsional_x.svg\", \"images/torsional_y.svg\")",
        ));
        pages.push(ReportPage::new(
            PageId::TorsionalTables,
            "Torsional Irregularity Tables",
            PageLayout::TwoTables,
            PageAvailability::WhenCalcDataPresent("torsional"),
            &["torsional.json"],
            &[],
            "#let tor = json(\"torsional.json\")\n#torsion-tables-pair-page([Torsional Irregularity Tables], tor)",
        ));
    }

    if calc.pier_shear_stress_wind.is_some() {
        pages.push(ReportPage::new(
            PageId::PierShearWindReview,
            "Pier Shear Wind Review",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("pier_shear_stress_wind"),
            &["pier_shear_wind.json"],
            &[
                "images/pier_shear_stress_wind_x.svg",
                "images/pier_shear_stress_wind_y.svg",
            ],
            "#let psw = json(\"pier_shear_wind.json\")\n#pier-shear-review-pair-page([Pier Shear Wind Review], psw, \"images/pier_shear_stress_wind_x.svg\", [Pier Shear Stress Ratio Wind — X Walls], \"images/pier_shear_stress_wind_y.svg\", [Pier Shear Stress Ratio Wind — Y Walls])",
        ));
        pages.push(ReportPage::new(
            PageId::PierShearWindTables,
            "Pier Shear Wind Tables",
            PageLayout::TwoTables,
            PageAvailability::WhenCalcDataPresent("pier_shear_stress_wind"),
            &["pier_shear_wind.json"],
            &[],
            "#let psw = json(\"pier_shear_wind.json\")\n#pier-shear-tables-pair-page([Pier Shear Wind Tables], psw)",
        ));
        pages.push(ReportPage::new(
            PageId::PierShearWindAverageReview,
            "Pier Shear Wind Average Review",
            PageLayout::OneChart,
            PageAvailability::WhenCalcDataPresent("pier_shear_stress_wind"),
            &["pier_shear_wind.json"],
            &["images/pier_shear_stress_wind_avg.svg"],
            "#pier-shear-average-review-page([Pier Shear Wind Average Review], \"pier_shear_wind.json\", \"images/pier_shear_stress_wind_avg.svg\")",
        ));
    }

    if calc.pier_shear_stress_seismic.is_some() {
        pages.push(ReportPage::new(
            PageId::PierShearSeismicReview,
            "Pier Shear Seismic Review",
            PageLayout::TwoCharts,
            PageAvailability::WhenCalcDataPresent("pier_shear_stress_seismic"),
            &["pier_shear_seismic.json"],
            &[
                "images/pier_shear_stress_seismic_x.svg",
                "images/pier_shear_stress_seismic_y.svg",
            ],
            "#let pss = json(\"pier_shear_seismic.json\")\n#pier-shear-review-pair-page([Pier Shear Seismic Review], pss, \"images/pier_shear_stress_seismic_x.svg\", [Pier Shear Stress Ratio Seismic — X Walls], \"images/pier_shear_stress_seismic_y.svg\", [Pier Shear Stress Ratio Seismic — Y Walls])",
        ));
        pages.push(ReportPage::new(
            PageId::PierShearSeismicTables,
            "Pier Shear Seismic Tables",
            PageLayout::TwoTables,
            PageAvailability::WhenCalcDataPresent("pier_shear_stress_seismic"),
            &["pier_shear_seismic.json"],
            &[],
            "#let pss = json(\"pier_shear_seismic.json\")\n#pier-shear-tables-pair-page([Pier Shear Seismic Tables], pss)",
        ));
        pages.push(ReportPage::new(
            PageId::PierShearSeismicAverageReview,
            "Pier Shear Seismic Average Review",
            PageLayout::OneChart,
            PageAvailability::WhenCalcDataPresent("pier_shear_stress_seismic"),
            &["pier_shear_seismic.json"],
            &["images/pier_shear_stress_seismic_avg.svg"],
            "#pier-shear-average-review-page([Pier Shear Seismic Average Review], \"pier_shear_seismic.json\", \"images/pier_shear_stress_seismic_avg.svg\")",
        ));
    }

    if calc.pier_axial_stress.is_some() {
        pages.push(ReportPage::new(
            PageId::PierAxialGravity,
            "Pier Axial - Gravity",
            PageLayout::OneChart,
            PageAvailability::WhenCalcDataPresent("pier_axial_stress"),
            &[],
            &["images/pier_axial_gravity.svg"],
            "#single-chart-page([Pier Axial - Gravity], \"images/pier_axial_gravity.svg\", [Pier Axial Stress — Gravity (ksi)])",
        ));
        pages.push(ReportPage::new(
            PageId::PierAxialWind,
            "Pier Axial - Wind",
            PageLayout::OneChart,
            PageAvailability::WhenCalcDataPresent("pier_axial_stress"),
            &[],
            &["images/pier_axial_wind.svg"],
            "#single-chart-page([Pier Axial - Wind], \"images/pier_axial_wind.svg\", [Pier Axial Stress — Wind (ksi)])",
        ));
        pages.push(ReportPage::new(
            PageId::PierAxialSeismic,
            "Pier Axial - Seismic",
            PageLayout::OneChart,
            PageAvailability::WhenCalcDataPresent("pier_axial_stress"),
            &[],
            &["images/pier_axial_seismic.svg"],
            "#single-chart-page([Pier Axial - Seismic], \"images/pier_axial_seismic.svg\", [Pier Axial Stress — Seismic (ksi)])",
        ));
    }

    if procedures::INCLUDE_CALC_PROCEDURE_PAGE {
        pages.push(ReportPage::new(
            PageId::VerificationExamples,
            "Verification Examples",
            PageLayout::Procedure,
            PageAvailability::WhenProcedurePageEnabled,
            &["torsional.json", "pier_shear_wind.json"],
            &[],
            "#calc-procedure-page()",
        ));
    }

    pages
}
