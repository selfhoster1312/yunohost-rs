use serde_json::{Map, Value};
// use snafu::prelude::*;

use std::str::FromStr;

use super::{error::*, ConfigPanel, ExcludeKey, FilterKey, OptionType, ALLOWED_EMPTY_TYPES};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportBullseyeContainer {
    #[serde(flatten)]
    pub fields: Map<String, Value>,
}

impl ExportBullseyeContainer {
    pub fn new() -> Self {
        Self { fields: Map::new() }
    }

    pub fn from_config_panel(
        cp: &ConfigPanel,
        filter_key: &FilterKey,
        exclude_key: &ExcludeKey,
    ) -> Result<Self, ConfigPanelError> {
        let saved_settings = cp.saved_settings()?;

        let mut export_container = Self::new();

        for (panel_id, panel) in &cp.container.panels {
            if !filter_key.matches_panel(panel_id) || exclude_key.excludes_panel(panel_id) {
                continue;
            }

            for (section_id, section) in &panel.sections {
                if !filter_key.matches_section(panel_id, section_id)
                    || exclude_key.excludes_section(panel_id, section_id)
                {
                    continue;
                }

                for (option_id, option) in &section.options {
                    if !filter_key.matches_option(panel_id, section_id, option_id)
                        || exclude_key.excludes_option(panel_id, section_id, option_id)
                    {
                        continue;
                    }

                    // So here we have "null" value in export mode for empty types like alert...
                    if let Ok(option_type) = OptionType::from_str(&option.option_type) {
                        if ALLOWED_EMPTY_TYPES.contains(&option_type) {
                            export_container
                                .fields
                                .insert(option_id.to_string(), Value::Null);
                            continue;
                        }
                    }

                    export_container.fields.insert(
                        option_id.to_string(),
                        ConfigPanel::value_or_default(&option_id, &option, &saved_settings).clone(),
                    );
                }
            }
        }

        Ok(export_container)
    }
}
