use anyhow::{Result, bail};

use crate::code_params::CodeParams;
use crate::output::{BaseReactionCheckRow, BaseShearDir, BaseShearOutput};
use crate::tables::base_reactions::BaseReactionRow;

pub fn run(rows: &[BaseReactionRow], params: &CodeParams) -> Result<BaseShearOutput> {
    if rows.is_empty() {
        bail!("No base reaction rows available");
    }

    let direction_x = build_direction(
        rows,
        &params.base_shear.rsa_case_x,
        &params.base_shear.elf_case_x,
        true,
        params,
    )?;
    let direction_y = build_direction(
        rows,
        &params.base_shear.rsa_case_y,
        &params.base_shear.elf_case_y,
        false,
        params,
    )?;

    let raw_rows = rows
        .iter()
        .filter(|row| !row.output_case.starts_with('~'))
        .filter(|row| !row.output_case.eq_ignore_ascii_case("Modal-Rizt"))
        .filter(|row| !row.output_case.eq_ignore_ascii_case("Modal-Eigen"))
        .filter(|row| !row.case_type.starts_with("LinMod"))
        .map(|row| BaseReactionCheckRow {
            output_case: row.output_case.clone(),
            case_type: row.case_type.clone(),
            step_type: row.step_type.clone(),
            step_number: row.step_number,
            fx_kip: row.fx_kip,
            fy_kip: row.fy_kip,
            fz_kip: row.fz_kip,
            mx_kip_ft: row.mx_kip_ft,
            my_kip_ft: row.my_kip_ft,
            mz_kip_ft: row.mz_kip_ft,
        })
        .collect();

    Ok(BaseShearOutput {
        rows: raw_rows,
        direction_x,
        direction_y,
    })
}

fn build_direction(
    rows: &[BaseReactionRow],
    rsa_case: &str,
    elf_case: &str,
    use_fx: bool,
    params: &CodeParams,
) -> Result<BaseShearDir> {
    let rsa = find_case_max(rows, rsa_case, use_fx)
        .ok_or_else(|| anyhow::anyhow!("Configured base shear case '{}' not found", rsa_case))?;
    let elf = find_case_max(rows, elf_case, use_fx)
        .ok_or_else(|| anyhow::anyhow!("Configured base shear case '{}' not found", elf_case))?;

    if elf == 0.0 {
        bail!("ELF base shear for case '{}' is zero", elf_case);
    }

    let ratio = rsa / elf;
    Ok(BaseShearDir {
        rsa_case: rsa_case.to_string(),
        elf_case: elf_case.to_string(),
        v_rsa: params.unit_context.qty_force(rsa),
        v_elf: params.unit_context.qty_force(elf),
        ratio,
        pass: ratio >= params.base_shear.rsa_scale_min,
    })
}

fn find_case_max(rows: &[BaseReactionRow], case: &str, use_fx: bool) -> Option<f64> {
    rows.iter()
        .filter(|row| row.output_case == case)
        .map(|row| {
            if use_fx {
                row.fx_kip.abs()
            } else {
                row.fy_kip.abs()
            }
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ext_db::config::Config;

    use crate::code_params::CodeParams;
    use crate::tables::base_reactions::load_base_reactions;

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
    fn base_shear_keeps_all_rows_and_computes_direction_ratios() {
        let rows = load_base_reactions(&fixture_dir()).unwrap();
        let config = fixture_config();
        let params = CodeParams::from_config(&config).unwrap();

        let output = run(&rows, &params).unwrap();
        assert!(output.rows.len() < rows.len());
        assert!((output.direction_x.ratio - 1.034_715_819_440_5).abs() < 1e-12);
        assert!((output.direction_y.ratio - 1.245_413_841_999_08).abs() < 1e-12);
        assert_eq!(output.direction_x.rsa_case, "RSA_X");
        assert_eq!(output.direction_y.elf_case, "ELF_Y");
        assert!(
            output
                .rows
                .iter()
                .all(|row| !row.output_case.starts_with('~'))
        );
        assert!(
            output
                .rows
                .iter()
                .all(|row| !row.output_case.starts_with("Modal-"))
        );
    }

    #[test]
    fn base_shear_errors_when_configured_case_missing() {
        let rows = load_base_reactions(&fixture_dir()).unwrap();
        let mut config = fixture_config();
        config.calc.base_shear.elf_case_x = Some("missing".into());
        let params = CodeParams::from_config(&config).unwrap();

        assert!(run(&rows, &params).is_err());
    }
}
