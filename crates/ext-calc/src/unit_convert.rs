use anyhow::{Result, bail};
use ext_db::config::Config;

use crate::output::Quantity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtabsPreset {
    KipFtF,
    KnMC,
}

impl EtabsPreset {
    pub fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "kip-ft-f" | "us_kip_ft" | "kip_ft" => Ok(Self::KipFtF),
            "kn-m-c" | "si_kn_m" | "kn_m" => Ok(Self::KnMC),
            other => bail!(
                "Unsupported unit preset '{other}'. Supported: kip-ft-F, kN-m-C"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnitContext {
    pub preset: EtabsPreset,
}

impl UnitContext {
    pub fn new(preset: EtabsPreset) -> Self {
        Self { preset }
    }

    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self::new(EtabsPreset::from_str(
            config.project.units_or_default(),
        )?))
    }

    pub fn force_to_kip(&self, value: f64) -> f64 {
        match self.preset {
            EtabsPreset::KipFtF => value,
            EtabsPreset::KnMC => value * 0.224_809,
        }
    }

    pub fn length_to_inch(&self, value: f64) -> f64 {
        match self.preset {
            EtabsPreset::KipFtF => value * 12.0,
            EtabsPreset::KnMC => value * 39.370_1,
        }
    }

    pub fn length_to_ft(&self, value: f64) -> f64 {
        match self.preset {
            EtabsPreset::KipFtF => value,
            EtabsPreset::KnMC => value * 3.280_84,
        }
    }

    pub fn stress_to_ksi(&self, value: f64) -> f64 {
        match self.preset {
            EtabsPreset::KipFtF => value / 144.0,
            EtabsPreset::KnMC => value * 0.000_145_038,
        }
    }

    pub fn force_label(&self) -> &'static str {
        match self.preset {
            EtabsPreset::KipFtF => "kip",
            EtabsPreset::KnMC => "kN",
        }
    }

    pub fn length_label(&self) -> &'static str {
        match self.preset {
            EtabsPreset::KipFtF => "ft",
            EtabsPreset::KnMC => "m",
        }
    }

    pub fn moment_label(&self) -> &'static str {
        match self.preset {
            EtabsPreset::KipFtF => "kip·ft",
            EtabsPreset::KnMC => "kN·m",
        }
    }

    pub fn qty_force(&self, kip: f64) -> Quantity {
        match self.preset {
            EtabsPreset::KipFtF => Quantity::new(kip, "kip"),
            EtabsPreset::KnMC => Quantity::new(kip / 0.224_809, "kN"),
        }
    }

    pub fn qty_area_in2(&self, in2: f64) -> Quantity {
        match self.preset {
            EtabsPreset::KipFtF => Quantity::new(in2 / 144.0, "ft²"),
            EtabsPreset::KnMC => Quantity::new(in2 * 0.000_645_16, "m²"),
        }
    }

    pub fn qty_length_disp(&self, ft: f64) -> Quantity {
        match self.preset {
            EtabsPreset::KipFtF => Quantity::new(ft * 12.0, "in"),
            EtabsPreset::KnMC => Quantity::new(ft * 304.8, "mm"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EtabsPreset, UnitContext};

    #[test]
    fn kip_ft_stress_to_ksi_8000psi() {
        let uc = UnitContext::new(EtabsPreset::KipFtF);
        assert!((uc.stress_to_ksi(1152.0) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn kn_m_stress_to_ksi_equivalent() {
        let uc = UnitContext::new(EtabsPreset::KnMC);
        assert!((uc.stress_to_ksi(55160.0) - 8.0).abs() < 0.01);
    }

    #[test]
    fn acv_calc_22x2ft() {
        let uc = UnitContext::new(EtabsPreset::KipFtF);
        let lw_in = uc.length_to_inch(22.0);
        let t_in = uc.length_to_inch(2.0);
        assert!((lw_in * t_in - 6336.0).abs() < 1e-6);
    }

    #[test]
    fn preset_parsing_accepts_variants() {
        assert!(EtabsPreset::from_str("kip-ft-F").is_ok());
        assert!(EtabsPreset::from_str("KIP-FT-F").is_ok());
        assert!(EtabsPreset::from_str("US_Kip_Ft").is_ok());
        assert!(EtabsPreset::from_str("kN-m-C").is_ok());
        assert!(EtabsPreset::from_str("SI_kN_m").is_ok());
        assert!(EtabsPreset::from_str("badunit").is_err());
    }
}
