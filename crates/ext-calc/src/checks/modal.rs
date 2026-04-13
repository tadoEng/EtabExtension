use anyhow::{Result, bail};

use crate::code_params::CodeParams;
use crate::output::{ModalModeRow, ModalOutput};
use crate::tables::modal::ModalParticipationRow;

pub fn run(rows: &[ModalParticipationRow], params: &CodeParams) -> Result<ModalOutput> {
    let mut filtered: Vec<&ModalParticipationRow> = rows
        .iter()
        .filter(|row| row.case_name == params.modal_case)
        .collect();

    if filtered.is_empty() {
        bail!(
            "Configured modal case '{}' not found in modal participation results",
            params.modal_case
        );
    }

    filtered.sort_by_key(|row| row.mode);

    let mode_reaching_ux = filtered
        .iter()
        .find(|row| row.sum_ux >= params.modal_threshold)
        .map(|row| row.mode);
    let mode_reaching_uy = filtered
        .iter()
        .find(|row| row.sum_uy >= params.modal_threshold)
        .map(|row| row.mode);

    let required_rows = mode_reaching_ux
        .into_iter()
        .chain(mode_reaching_uy)
        .max()
        .and_then(|mode| usize::try_from(mode).ok())
        .unwrap_or(0);
    let display_limit = required_rows.max(params.modal_display_limit);

    let display_rows = filtered
        .iter()
        .take(display_limit)
        .map(|row| ModalModeRow {
            case: row.case_name.clone(),
            mode: row.mode,
            period: row.period_sec,
            ux: row.ux,
            uy: row.uy,
            sum_ux: row.sum_ux,
            sum_uy: row.sum_uy,
            rz: row.rz,
            sum_rz: row.sum_rz,
        })
        .collect();

    Ok(ModalOutput {
        rows: display_rows,
        threshold: params.modal_threshold,
        mode_reaching_ux,
        mode_reaching_uy,
        pass: mode_reaching_ux.is_some() && mode_reaching_uy.is_some(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ext_db::config::Config;

    use crate::code_params::CodeParams;
    use crate::tables::modal::load_modal_participating_mass_ratios;

    use super::run;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("results_realistic")
    }

    fn fixture_config() -> Config {
        Config::load(&fixture_dir()).unwrap()
    }

    #[test]
    fn modal_check_limits_rows_and_finds_threshold_modes() {
        let rows = load_modal_participating_mass_ratios(&fixture_dir()).unwrap();
        let config = fixture_config();
        let params = CodeParams::from_config(&config).unwrap();

        let output = run(&rows, &params).unwrap();
        assert_eq!(output.mode_reaching_ux, Some(15));
        assert_eq!(output.mode_reaching_uy, Some(7));
        assert_eq!(output.rows.len(), 23);
        assert!(
            output
                .rows
                .windows(2)
                .all(|pair| pair[0].mode <= pair[1].mode)
        );
        assert_eq!(output.rows.first().map(|row| row.mode), Some(1));
        assert_eq!(output.rows.last().map(|row| row.mode), Some(23));
    }

    #[test]
    fn modal_check_errors_when_case_missing() {
        let rows = load_modal_participating_mass_ratios(&fixture_dir()).unwrap();
        let mut config = fixture_config();
        config.calc.modal_case = Some("missing-case".into());
        let params = CodeParams::from_config(&config).unwrap();

        assert!(run(&rows, &params).is_err());
    }
}
