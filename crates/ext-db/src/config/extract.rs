// ext-db::config::extract — [extract] section of config.toml
//
// TableSelections drives what extract-results actually requests from ETABS.
// None   = skip that table entirely
// ["*"]  = request ALL items from the model
// ["X"]  = request exactly those named items

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ExtractConfig {
    /// Override the default unit preset for extraction only.
    pub units: Option<String>,

    #[serde(default)]
    pub tables: TableSelections,
}

impl ExtractConfig {
    pub fn merge(self, other: Self) -> Self {
        Self {
            units: other.units.or(self.units),
            tables: self.tables.merge(other.tables),
        }
    }
}

/// Maps to the ETABS result tables supported by the extraction contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSelections {
    pub story_definitions: Option<TableConfig>,
    pub pier_section_properties: Option<TableConfig>,
    pub base_reactions: Option<TableConfig>,
    pub story_forces: Option<TableConfig>,
    pub joint_drifts: Option<TableConfig>,
    pub pier_forces: Option<TableConfig>,
    pub modal_participating_mass_ratios: Option<TableConfig>,
    pub group_assignments: Option<TableConfig>,
    pub material_properties_concrete_data: Option<TableConfig>,
    pub material_list_by_story: Option<TableConfig>,
}

impl TableSelections {
    /// Merge other over self — other's Some values win per table.
    pub fn merge(self, other: Self) -> Self {
        Self {
            story_definitions: other.story_definitions.or(self.story_definitions),
            pier_section_properties: other
                .pier_section_properties
                .or(self.pier_section_properties),
            base_reactions: other.base_reactions.or(self.base_reactions),
            story_forces: other.story_forces.or(self.story_forces),
            joint_drifts: other.joint_drifts.or(self.joint_drifts),
            pier_forces: other.pier_forces.or(self.pier_forces),
            modal_participating_mass_ratios: other
                .modal_participating_mass_ratios
                .or(self.modal_participating_mass_ratios),
            group_assignments: other.group_assignments.or(self.group_assignments),
            material_properties_concrete_data: other
                .material_properties_concrete_data
                .or(self.material_properties_concrete_data),
            material_list_by_story: other.material_list_by_story.or(self.material_list_by_story),
        }
    }

    /// Returns true if no tables are selected (all None).
    pub fn is_empty(&self) -> bool {
        self.story_definitions.is_none()
            && self.pier_section_properties.is_none()
            && self.base_reactions.is_none()
            && self.story_forces.is_none()
            && self.joint_drifts.is_none()
            && self.pier_forces.is_none()
            && self.modal_participating_mass_ratios.is_none()
            && self.group_assignments.is_none()
            && self.material_properties_concrete_data.is_none()
            && self.material_list_by_story.is_none()
    }
}

/// Per-table selection filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableConfig {
    pub load_cases: Option<Vec<String>>,
    pub load_combos: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
    pub field_keys: Option<Vec<String>>,
}
