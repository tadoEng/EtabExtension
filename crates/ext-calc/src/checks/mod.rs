pub mod base_reaction;
pub mod displacement_wind;
pub mod drift_seismic;
pub mod drift_wind;
pub mod modal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckSelection {
    pub modal: bool,
    pub base_shear: bool,
    pub drift_wind: bool,
    pub drift_seismic: bool,
    pub displacement_wind: bool,
    pub torsional: bool,
    pub pier_shear_wind: bool,
    pub pier_shear_seismic: bool,
    pub pier_axial: bool,
}

impl Default for CheckSelection {
    fn default() -> Self {
        Self {
            modal: true,
            base_shear: true,
            drift_wind: true,
            drift_seismic: true,
            displacement_wind: true,
            torsional: false,
            pier_shear_wind: true,
            pier_shear_seismic: true,
            pier_axial: true,
        }
    }
}
