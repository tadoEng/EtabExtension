use anyhow::Result;
use ext_db::config::Config;

use crate::checks::CheckSelection;
use crate::unit_convert::UnitContext;

#[derive(Debug, Clone)]
pub struct CodeParams {
    pub code: String,
    pub occupancy_category: String,
    pub modal_case: String,
    pub drift_tracking_groups: Vec<String>,
    pub check_selection: CheckSelection,
    pub unit_context: UnitContext,
}

impl CodeParams {
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self {
            code: config.calc.code_or_default().to_string(),
            occupancy_category: config.calc.occupancy_or_default().to_string(),
            modal_case: config.calc.modal_case_or_default().to_string(),
            drift_tracking_groups: config.calc.drift_groups_or_default(),
            check_selection: CheckSelection::default(),
            unit_context: UnitContext::from_config(config)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use ext_db::config::Config;

    use super::CodeParams;

    #[test]
    fn code_params_pick_up_shared_calc_and_local_units() {
        let mut config = Config::default();
        config.calc.code = Some("ACI318-14".into());
        config.calc.modal_case = Some("Modal-Rizt".into());
        config.project.units = Some("kip-ft-F".into());

        let params = CodeParams::from_config(&config).unwrap();
        assert_eq!(params.code, "ACI318-14");
        assert_eq!(params.modal_case, "Modal-Rizt");
        assert_eq!(params.unit_context.force_label(), "kip");
    }
}
