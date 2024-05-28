use snafu::prelude::*;

use std::str::FromStr;

use super::{
    error::*, field_i18n_single, ConfigPanel, ExcludeKey, FilterKey, Map, OptionType,
    ALLOWED_EMPTY_TYPES,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedClassicContainer {
    #[serde(flatten)]
    pub fields: Map<String, AppliedClassicValue>,
}

impl AppliedClassicContainer {
    pub fn new() -> Self {
        Self { fields: Map::new() }
    }

    /// The values are normalized/humanized for use in get_multi
    pub fn from_config_panel(
        cp: &ConfigPanel,
        filter_key: &FilterKey,
        exclude_key: &ExcludeKey,
    ) -> Result<AppliedClassicContainer, ConfigPanelError> {
        let saved_settings = cp.saved_settings()?;

        let mut classic_container = AppliedClassicContainer::new();

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

                    // Maybe we should skip this option because it doesn't have an actual value?
                    // if let Some(bind) = option.get("bind").map(|x| x.as_str()).flatten() {
                    //     // TODO: what is this?
                    //     continue;
                    // }

                    let ask = field_i18n_single(
                        "ask",
                        &option_id,
                        &option,
                        cp.container
                            .i18n_key
                            .as_ref()
                            .map(|x| format!("{}_{}", x, option_id)),
                    );

                    if let Ok(option_type) = OptionType::from_str(&option.option_type) {
                        // Apparently at least for alert we have the ask to insert, but no value...
                        // TODO: Is that true for other ALLOWED_EMPTY_TYPES?
                        if ALLOWED_EMPTY_TYPES.contains(&option_type) {
                            classic_container.fields.insert(
                                format!("{}.{}.{}", panel_id, section_id, option_id),
                                AppliedClassicValue::new(ask, None),
                            );
                            continue;
                        }
                    }

                    let value = ConfigPanel::value_or_default(&option_id, &option, &saved_settings);

                    let option_type = OptionType::from_str(&option.option_type).context(
                        OptionTypeWrongSnafu {
                            option_id: option_id.to_string(),
                            option_type: option.option_type.to_string(),
                        },
                    )?;
                    let value = ConfigPanel::humanize(&option_type, value);
                    classic_container.fields.insert(
                        format!("{}.{}.{}", panel_id, section_id, option_id),
                        AppliedClassicValue::new(ask, Some(value)),
                    );
                }
            }
        }

        Ok(classic_container)
    }
}

/// Once we have applied settings and translated stuff, only ask/value remain in the compact view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedClassicValue {
    pub ask: String,
    // TODO: why is everything always a string???
    // Actually, for type="alert", we have a "ask" but no value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl AppliedClassicValue {
    pub fn new(ask: String, value: Option<String>) -> Self {
        Self { ask, value }
    }
}
