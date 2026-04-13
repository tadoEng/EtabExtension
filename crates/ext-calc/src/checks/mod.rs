pub mod base_reaction;
pub mod displacement_wind;
pub mod drift_seismic;
pub mod drift_wind;
pub mod modal;
pub mod pier_axial;
pub mod pier_shear_stress;
pub mod story_forces;
pub mod torsional;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckSelection {
    pub modal: bool,
    pub base_reactions: bool,
    pub story_forces: bool,
    pub drift_wind: bool,
    pub drift_seismic: bool,
    pub displacement_wind: bool,
    pub torsional: bool,
    pub pier_shear_stress_wind: bool,
    pub pier_shear_stress_seismic: bool,
    pub pier_axial_stress: bool,
}

impl Default for CheckSelection {
    fn default() -> Self {
        Self {
            modal: true,
            base_reactions: true,
            story_forces: true,
            drift_wind: true,
            drift_seismic: true,
            displacement_wind: true,
            torsional: false,
            pier_shear_stress_wind: true,
            pier_shear_stress_seismic: true,
            pier_axial_stress: true,
        }
    }
}
