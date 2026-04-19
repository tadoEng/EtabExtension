use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;
use typst::foundations::Bytes;

use ext_calc::output::CalcOutput;

use crate::theme::PageTheme;

mod base_reactions;
mod displacement;
mod drift;
mod format;
mod modal;
mod ordering;
mod pier_axial;
mod pier_shear;
mod story_forces;
mod summary;
mod torsional;

use base_reactions::build_base_reactions;
use displacement::{DisplacementReportData, build_displacement_dir};
use drift::{DriftReportData, build_drift_dir};
use modal::build_modal;
use pier_axial::build_pier_axial;
use pier_shear::{build_pier_shear, build_unsupported_pier_shear_report_data};
use story_forces::build_story_forces;
use summary::build_summary;
use torsional::{build_torsional, default_torsional_report_data};

#[cfg(test)]
use ordering::{compare_pier_labels, is_default_pier_label, ordered_unique};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
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
    pub sheet_prefix: String,
}

pub struct ReportData {
    pub files: HashMap<PathBuf, Bytes>,
}

impl ReportData {
    pub fn from_calc(
        calc: &CalcOutput,
        project: &ReportProjectMeta,
        theme: &PageTheme,
        svg_map: HashMap<String, String>,
    ) -> Result<Self> {
        let mut files = HashMap::new();

        files.insert(pb("theme.json"), to_json(theme)?);
        files.insert(pb("project.json"), to_json(project)?);
        files.insert(pb("summary.json"), to_json(&build_summary(calc))?);

        if let Some(v) = &calc.modal {
            files.insert(pb("modal.json"), to_json(&build_modal(v))?);
        }
        if let Some(v) = &calc.base_reactions {
            files.insert(
                pb("base_reactions.json"),
                to_json(&build_base_reactions(v))?,
            );
        }
        if let Some(v) = &calc.story_forces {
            files.insert(pb("story_forces.json"), to_json(&build_story_forces(v))?);
        }
        if let Some(v) = &calc.drift_wind {
            files.insert(
                pb("drift_wind.json"),
                to_json(&DriftReportData {
                    x: build_drift_dir(&v.x),
                    y: build_drift_dir(&v.y),
                })?,
            );
        }
        if let Some(v) = &calc.drift_seismic {
            files.insert(
                pb("drift_seismic.json"),
                to_json(&DriftReportData {
                    x: build_drift_dir(&v.x),
                    y: build_drift_dir(&v.y),
                })?,
            );
        }
        if let Some(v) = &calc.displacement_wind {
            files.insert(
                pb("displacement_wind.json"),
                to_json(&DisplacementReportData {
                    x: build_displacement_dir(&v.x),
                    y: build_displacement_dir(&v.y),
                })?,
            );
        }
        let torsional_data = calc
            .torsional
            .as_ref()
            .map(build_torsional)
            .unwrap_or_else(default_torsional_report_data);
        files.insert(pb("torsional.json"), to_json(&torsional_data)?);
        if let Some(v) = &calc.pier_axial_stress {
            files.insert(pb("pier_axial_stress.json"), to_json(&build_pier_axial(v))?);
        }
        let pier_shear_wind_data = calc
            .pier_shear_stress_wind
            .as_ref()
            .map(build_pier_shear)
            .unwrap_or_else(|| {
                build_unsupported_pier_shear_report_data("No wind pier shear data available.")
            });
        files.insert(pb("pier_shear_wind.json"), to_json(&pier_shear_wind_data)?);
        let pier_shear_seismic_data = calc
            .pier_shear_stress_seismic
            .as_ref()
            .map(build_pier_shear)
            .unwrap_or_else(|| {
                build_unsupported_pier_shear_report_data("No seismic pier shear data available.")
            });
        files.insert(
            pb("pier_shear_seismic.json"),
            to_json(&pier_shear_seismic_data)?,
        );

        for (name, svg) in svg_map {
            files.insert(PathBuf::from(&name), Bytes::new(svg.into_bytes()));
        }

        Ok(Self { files })
    }
}

fn pb(s: &str) -> PathBuf {
    PathBuf::from(s)
}

fn to_json<T: Serialize>(v: &T) -> Result<Bytes> {
    Ok(Bytes::new(serde_json::to_vec(v)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::TABLOID_LANDSCAPE;
    use ext_calc::CalcRunner;
    use ext_calc::code_params::CodeParams;
    use ext_db::config::Config;

    fn fixture_calc_output() -> CalcOutput {
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ext-calc/tests/fixtures/results_realistic");
        let path = fixture_dir.join("calc_output.json");
        if path.exists() {
            let text = std::fs::read_to_string(path).unwrap();
            serde_json::from_str(&text).unwrap()
        } else {
            let config = Config::load(&fixture_dir).unwrap();
            let params = CodeParams::from_config(&config).unwrap();
            CalcRunner::run_all(
                fixture_dir.as_path(),
                fixture_dir.as_path(),
                &params,
                "fixture",
                "main",
            )
            .unwrap()
        }
    }

    fn sample_project() -> ReportProjectMeta {
        ReportProjectMeta {
            project_name: "Project Test".to_string(),
            project_number: "v1".to_string(),
            reference: "main/v1".to_string(),
            engineer: "Preview".to_string(),
            checker: "Preview".to_string(),
            date: "2026-04-15".to_string(),
            subject: "Fixture report".to_string(),
            scale: "NTS".to_string(),
            revision: "0".to_string(),
            sheet_prefix: "SK".to_string(),
        }
    }

    #[test]
    fn report_data_includes_story_forces_json_when_present() {
        let calc = fixture_calc_output();
        assert!(calc.story_forces.is_some());
        let report_data =
            ReportData::from_calc(&calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new())
                .unwrap();
        assert!(
            report_data
                .files
                .contains_key(&PathBuf::from("story_forces.json"))
        );
    }

    #[test]
    fn summary_json_includes_calc_code() {
        let calc = fixture_calc_output();
        let report_data =
            ReportData::from_calc(&calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new())
                .unwrap();
        let bytes = report_data
            .files
            .get(&PathBuf::from("summary.json"))
            .expect("summary.json must exist");
        let value: serde_json::Value = serde_json::from_slice(bytes.as_slice()).unwrap();
        assert_eq!(
            value.get("code").and_then(|v| v.as_str()),
            Some(calc.meta.code.as_str())
        );
    }

    #[test]
    fn base_reactions_json_excludes_unscoped_case_types() {
        let calc = fixture_calc_output();
        let report_data =
            ReportData::from_calc(&calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new())
                .unwrap();
        let bytes = report_data
            .files
            .get(&PathBuf::from("base_reactions.json"))
            .expect("base_reactions.json must exist");
        let value: serde_json::Value = serde_json::from_slice(bytes.as_slice()).unwrap();
        let rows = value
            .get("rows")
            .and_then(|node| node.as_array())
            .expect("rows should be array");
        assert!(rows.iter().all(|row| {
            let case_type = row
                .get("case-type")
                .and_then(|node| node.as_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            case_type != "combination" && case_type != "linmodritz" && case_type != "eigen"
        }));
    }

    #[test]
    fn pier_shear_wind_json_includes_directional_matrix_payloads() {
        let calc = fixture_calc_output();
        let wind = calc
            .pier_shear_stress_wind
            .as_ref()
            .expect("fixture should include wind pier shear");
        let report_data =
            ReportData::from_calc(&calc, &sample_project(), &TABLOID_LANDSCAPE, HashMap::new())
                .unwrap();
        let bytes = report_data
            .files
            .get(&PathBuf::from("pier_shear_wind.json"))
            .expect("pier_shear_wind.json must exist");
        let value: serde_json::Value = serde_json::from_slice(bytes.as_slice()).unwrap();

        let assert_direction_matrix = |matrix_key: &str, direction: &str| {
            let matrix_node = value
                .get(matrix_key)
                .unwrap_or_else(|| panic!("{matrix_key} should exist"));

            let json_levels = matrix_node
                .get("levels")
                .and_then(|node| node.as_array())
                .expect("levels should be array")
                .iter()
                .map(|node| node.as_str().unwrap_or_default().to_string())
                .collect::<Vec<_>>();
            let json_piers = matrix_node
                .get("piers")
                .and_then(|node| node.as_array())
                .expect("piers should be array")
                .iter()
                .map(|node| node.as_str().unwrap_or_default().to_string())
                .collect::<Vec<_>>();

            let filtered = wind
                .per_pier
                .iter()
                .filter(|row| {
                    !is_default_pier_label(&row.pier)
                        && row.wall_direction.eq_ignore_ascii_case(direction)
                })
                .collect::<Vec<_>>();

            let expected_levels = wind
                .story_order
                .iter()
                .filter(|story| filtered.iter().any(|row| row.story == **story))
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(json_levels, expected_levels);

            let mut expected_piers = ordered_unique(filtered.iter().map(|row| row.pier.clone()));
            expected_piers.sort_by(|a, b| compare_pier_labels(a, b));
            assert_eq!(json_piers, expected_piers);

            let mut expected_values: HashMap<(String, String), f64> = HashMap::new();
            for row in filtered {
                let key = (row.story.clone(), row.pier.clone());
                let entry = expected_values.entry(key).or_insert(0.0);
                *entry = entry.max(row.stress_ratio);
            }

            let json_matrix = matrix_node
                .get("matrix-ratio")
                .and_then(|node| node.as_array())
                .expect("matrix-ratio should be array");
            assert_eq!(json_matrix.len(), expected_levels.len());

            for (row_idx, level) in expected_levels.iter().enumerate() {
                let json_row = json_matrix[row_idx]
                    .as_array()
                    .expect("matrix row should be an array");
                assert_eq!(json_row.len(), expected_piers.len());
                for (col_idx, pier) in expected_piers.iter().enumerate() {
                    let expected = expected_values.get(&(level.clone(), pier.clone())).copied();
                    let actual = if json_row[col_idx].is_null() {
                        None
                    } else {
                        Some(
                            json_row[col_idx]
                                .as_f64()
                                .expect("matrix ratio should be number or null"),
                        )
                    };
                    match (expected, actual) {
                        (None, None) => {}
                        (Some(exp), Some(act)) => {
                            assert!(
                                (exp - act).abs() <= 1.0e-9,
                                "matrix mismatch for direction={direction}, story={level}, pier={pier}: expected={exp}, actual={act}"
                            );
                        }
                        _ => {
                            panic!(
                                "presence mismatch for direction={direction}, story={level}, pier={pier}"
                            );
                        }
                    }
                }
            }
        };

        assert_direction_matrix("x-matrix", "X");
        assert_direction_matrix("y-matrix", "Y");
    }
}
