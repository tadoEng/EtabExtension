use anyhow::{Result, bail};
use ext_db::config::Config;

use crate::output::Quantity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtabsPreset {
    KipFtF,
    KipInF,
    KnMC,
}

use std::str::FromStr;

impl FromStr for EtabsPreset {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "kip-ft-f" | "us_kip_ft" | "kip_ft" => Ok(Self::KipFtF),
            "kip-in-f" | "us_kip_in" | "kip_in" | "kip/in/f" | "kip-in" => Ok(Self::KipInF),
            "kn-m-c" | "si_kn_m" | "kn_m" => Ok(Self::KnMC),
            other => {
                bail!("Unsupported unit preset '{other}'. Supported: kip-ft-F, kip-in-F, kN-m-C")
            }
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
            EtabsPreset::KipInF => value,
            EtabsPreset::KnMC => value * 0.224_809,
        }
    }

    pub fn length_to_inch(&self, value: f64) -> f64 {
        match self.preset {
            EtabsPreset::KipFtF => value * 12.0,
            EtabsPreset::KipInF => value,
            EtabsPreset::KnMC => value * 39.370_1,
        }
    }

    pub fn length_to_ft(&self, value: f64) -> f64 {
        match self.preset {
            EtabsPreset::KipFtF => value,
            EtabsPreset::KipInF => value / 12.0,
            EtabsPreset::KnMC => value * 3.280_84,
        }
    }

    pub fn stress_to_ksi(&self, value: f64) -> f64 {
        match self.preset {
            EtabsPreset::KipFtF => value / 144.0,
            EtabsPreset::KipInF => value,
            EtabsPreset::KnMC => value * 0.000_145_038,
        }
    }

    pub fn force_label(&self) -> &'static str {
        match self.preset {
            EtabsPreset::KipFtF | EtabsPreset::KipInF => "kip",
            EtabsPreset::KnMC => "kN",
        }
    }

    pub fn length_label(&self) -> &'static str {
        match self.preset {
            EtabsPreset::KipFtF => "ft",
            EtabsPreset::KipInF => "in",
            EtabsPreset::KnMC => "m",
        }
    }

    pub fn moment_label(&self) -> &'static str {
        match self.preset {
            EtabsPreset::KipFtF => "kip·ft",
            EtabsPreset::KipInF => "kip·in",
            EtabsPreset::KnMC => "kN·m",
        }
    }

    pub fn qty_force(&self, kip: f64) -> Quantity {
        match self.preset {
            EtabsPreset::KipFtF | EtabsPreset::KipInF => Quantity::new(kip, "kip"),
            EtabsPreset::KnMC => Quantity::new(kip / 0.224_809, "kN"),
        }
    }

    pub fn qty_area_in2(&self, in2: f64) -> Quantity {
        match self.preset {
            EtabsPreset::KipFtF => Quantity::new(in2 / 144.0, "ft²"),
            EtabsPreset::KipInF => Quantity::new(in2, "in²"),
            EtabsPreset::KnMC => Quantity::new(in2 * 0.000_645_16, "m²"),
        }
    }

    pub fn qty_length_disp(&self, ft: f64) -> Quantity {
        match self.preset {
            EtabsPreset::KipFtF | EtabsPreset::KipInF => Quantity::new(ft * 12.0, "in"),
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
        assert!("kip-ft-F".parse::<EtabsPreset>().is_ok());
        assert!("KIP-FT-F".parse::<EtabsPreset>().is_ok());
        assert!("US_Kip_Ft".parse::<EtabsPreset>().is_ok());
        assert!("kip-in-F".parse::<EtabsPreset>().is_ok());
        assert!("US_Kip_In".parse::<EtabsPreset>().is_ok());
        assert!("kN-m-C".parse::<EtabsPreset>().is_ok());
        assert!("SI_kN_m".parse::<EtabsPreset>().is_ok());
        assert!("badunit".parse::<EtabsPreset>().is_err());
    }

    #[test]
    fn kip_in_maps_lengths_and_stress_directly() {
        let uc = UnitContext::new(EtabsPreset::KipInF);
        assert!((uc.length_to_inch(22.0) - 22.0).abs() < 1e-9);
        assert!((uc.length_to_ft(24.0) - 2.0).abs() < 1e-9);
        assert!((uc.stress_to_ksi(8.0) - 8.0).abs() < 1e-9);
        assert_eq!(uc.moment_label(), "kip·in");
    }
}
