use anyhow::{Result, bail};

use crate::tables::base_reactions::BaseReactionRow;
use crate::{
    code_params::CodeParams,
    output::{BaseReactionCheckRow, BaseReactionDir, BaseReactionsOutput, Quantity},
};

pub fn run(reactions: &[BaseReactionRow], params: &CodeParams) -> Result<BaseReactionsOutput> {
    let rows = reactions
        .iter()
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
        .collect::<Vec<_>>();

    Ok(BaseReactionsOutput {
        rows,
        direction_x: build_direction(
            reactions,
            &params.base_reactions.rsa_case_x,
            &params.base_reactions.elf_case_x,
            params.base_reactions.rsa_scale_min,
            true,
            params,
        )?,
        direction_y: build_direction(
            reactions,
            &params.base_reactions.rsa_case_y,
            &params.base_reactions.elf_case_y,
            params.base_reactions.rsa_scale_min,
            false,
            params,
        )?,
    })
}

fn build_direction(
    reactions: &[BaseReactionRow],
    rsa_case: &str,
    elf_case: &str,
    rsa_scale_min: f64,
    use_x_force: bool,
    params: &CodeParams,
) -> Result<BaseReactionDir> {
    let component = if use_x_force { "FX" } else { "FY" };
    let v_rsa = max_abs_force_for_case(reactions, rsa_case, use_x_force).ok_or_else(|| {
        anyhow::anyhow!(
            "Configured base reaction case '{}' not found in base reactions ({component})",
            rsa_case
        )
    })?;
    let v_elf = max_abs_force_for_case(reactions, elf_case, use_x_force).ok_or_else(|| {
        anyhow::anyhow!(
            "Configured base reaction case '{}' not found in base reactions ({component})",
            elf_case
        )
    })?;

    if v_elf <= f64::EPSILON {
        bail!(
            "Configured ELF base reaction case '{}' has zero {} demand; cannot compute RSA scale ratio",
            elf_case,
            component
        );
    }

    let ratio = v_rsa / v_elf;
    Ok(BaseReactionDir {
        rsa_case: rsa_case.to_string(),
        elf_case: elf_case.to_string(),
        v_rsa: Quantity::new(v_rsa, params.unit_context.force_label()),
        v_elf: Quantity::new(v_elf, params.unit_context.force_label()),
        ratio,
        pass: ratio >= rsa_scale_min,
    })
}

fn max_abs_force_for_case(
    reactions: &[BaseReactionRow],
    output_case: &str,
    use_x_force: bool,
) -> Option<f64> {
    reactions
        .iter()
        .filter(|row| row.output_case == output_case)
        .map(|row| {
            if use_x_force {
                row.fx_kip.abs()
            } else {
                row.fy_kip.abs()
            }
        })
        .max_by(|a, b| a.total_cmp(b))
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::{code_params::CodeParams, tables::base_reactions::BaseReactionRow};

    fn sample_row(case: &str, fx: f64, fy: f64, fz: f64) -> BaseReactionRow {
        BaseReactionRow {
            output_case: case.to_string(),
            case_type: "LinStatic".to_string(),
            step_type: String::new(),
            step_number: None,
            fx_kip: fx,
            fy_kip: fy,
            fz_kip: fz,
            mx_kip_ft: 0.0,
            my_kip_ft: 0.0,
            mz_kip_ft: 0.0,
            x_ft: 0.0,
            y_ft: 0.0,
            z_ft: 0.0,
        }
    }

    #[test]
    fn base_reactions_compute_directional_rsa_ratios() {
        let params = CodeParams::for_testing();
        let rows = vec![
            sample_row("ELF_X", 120.0, 10.0, 800.0),
            sample_row("RSA_X", 150.0, 15.0, 900.0),
            sample_row("ELF_Y", 8.0, 100.0, 780.0),
            sample_row("RSA_Y", 10.0, 130.0, 840.0),
        ];

        let output = run(&rows, &params).unwrap();

        assert_eq!(output.rows.len(), 4);
        assert!((output.direction_x.ratio - 1.25).abs() < 1e-9);
        assert!((output.direction_y.ratio - 1.30).abs() < 1e-9);
        assert!(output.direction_x.pass);
        assert!(output.direction_y.pass);
    }
}
