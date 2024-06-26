use serde_json::Value;

use std::str::FromStr;

use super::{
    field_i18n_single, field_i18n_single_optional_bullseye_englishname, ApplyAction,
    BullseyePanelVersion, ConfigPanel, ConfigPanelError, EnglishName, ExcludeKey, FilterKey, Map,
    OptionToml, PanelToml, SectionToml, ALLOWED_EMPTY_TYPES,
};
use crate::helpers::{distro::DebianRelease, form::OptionType};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BullseyeFullContainer {
    pub version: BullseyePanelVersion,
    #[serde(rename = "i18n")]
    pub i18n_key: String,
    pub panels: Vec<BullseyeFullPanel>,
}

impl BullseyeFullContainer {
    pub fn new(i18n_key: &str) -> Self {
        Self {
            version: BullseyePanelVersion::V1_0,
            i18n_key: i18n_key.to_string(),
            panels: Vec::new(),
        }
    }

    pub fn from_config_panel(
        cp: &ConfigPanel,
        filter_key: &FilterKey,
        exclude_key: &ExcludeKey,
    ) -> Result<Self, ConfigPanelError> {
        let saved_settings = cp.saved_settings()?;

        // TODO: so is i18n_key not optional after all?
        let mut full_container = Self::new(&cp.container.i18n_key.clone().unwrap());

        for (panel_id, panel) in &cp.container.panels {
            if !filter_key.matches_panel(panel_id) || exclude_key.excludes_panel(panel_id) {
                continue;
            }

            let mut full_panel = BullseyeFullPanel::from_panel_with_id(panel, panel_id);

            for (section_id, section) in &panel.sections {
                if !filter_key.matches_section(panel_id, section_id)
                    || exclude_key.excludes_section(panel_id, section_id)
                {
                    continue;
                }
                let mut full_section =
                    BullseyeFullSection::from_section_with_id(&section, &section_id);

                for (option_id, option) in &section.options {
                    if !filter_key.matches_option(panel_id, section_id, option_id)
                        || exclude_key.excludes_option(panel_id, section_id, option_id)
                    {
                        continue;
                    }

                    if let Ok(option_type) = OptionType::from_str(&option.option_type) {
                        if ALLOWED_EMPTY_TYPES.contains(&option_type) {
                            let alert_option = AppliedAllowedEmptyOption::from_option_with_id(
                                &option,
                                &option_id,
                                cp.container.i18n_key.as_ref(),
                            );
                            full_section
                                .options
                                .push(MaybeEmptyBullseyeOption::NoValue(alert_option));
                            continue;
                        }
                    }

                    let full_option = BullseyeFullOption::from_option_with_id(
                        &option,
                        &option_id,
                        cp.container.i18n_key.as_ref(),
                        &saved_settings,
                    );
                    full_section
                        .options
                        .push(MaybeEmptyBullseyeOption::SomeValue(full_option));
                }

                full_panel.sections.push(full_section);
            }

            full_container.panels.push(full_panel);
        }

        Ok(full_container)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BullseyeFullPanel {
    pub actions: ApplyAction,
    pub id: String,
    pub name: EnglishName,
    pub sections: Vec<BullseyeFullSection>,
    pub services: Vec<String>,
}

impl BullseyeFullPanel {
    pub fn from_panel_with_id(panel: &PanelToml, id: &str) -> BullseyeFullPanel {
        BullseyeFullPanel {
            actions: ApplyAction::default(),
            id: id.to_string(),
            name: EnglishName::new(&panel.name),
            sections: Vec::new(),
            services: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BullseyeFullSection {
    pub id: String,
    #[serde(default)]
    pub is_action_section: bool,
    #[serde(default)]
    pub optional: bool,
    pub name: EnglishName,
    pub services: Vec<String>,
    pub options: Vec<MaybeEmptyBullseyeOption>,
}

impl BullseyeFullSection {
    pub fn from_section_with_id(section: &SectionToml, id: &str) -> BullseyeFullSection {
        BullseyeFullSection {
            id: id.to_string(),
            is_action_section: false,
            optional: section.optional.unwrap_or(true),
            name: EnglishName::new(&section.name),
            services: Vec::new(),
            options: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BullseyeFullOption {
    #[serde(flatten)]
    /// Extra options defined by the [`OptionType`].
    // It's the first field in the struct because otherwise flattening those entries
    // will override fields that have already been set with those inside the Map.
    pub fields: Map<String, Value>,
    pub ask: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<Value>,
    pub id: String,
    pub name: String,
    pub optional: bool,
    pub current_value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(rename = "type")]
    pub option_type: String,
    pattern: Option<String>,
}

impl BullseyeFullOption {
    pub fn from_option_with_id(
        option: &OptionToml,
        id: &String,
        container_i18n_key: Option<&String>,
        saved_settings: &Map<String, Value>,
    ) -> BullseyeFullOption {
        let ask = field_i18n_single(
            "ask",
            id,
            option,
            container_i18n_key.map(|x| format!("{}_{}", x, id)),
        );

        let help = field_i18n_single_optional_bullseye_englishname(
            "help",
            option,
            container_i18n_key.map(|x| format!("{}_{}_help", x, id)),
        );

        // In full output, the OptionType may set additional defaults
        let fields = if let Ok(option_type) = OptionType::from_str(&option.option_type) {
            if let Some(type_defaults) = option_type
                .to_option_type()
                .full_extra_fields(id, DebianRelease::Bullseye)
            {
                let mut type_defaults = type_defaults.into_iter().collect::<Map<String, Value>>();
                type_defaults.extend(option.fields.clone());
                type_defaults
            } else {
                option.fields.clone()
            }
        } else {
            option.fields.clone()
        };

        // Apparently when default is "" in bullseye branch, it's converted to null
        let default = if let Some(val) = &option.default {
            if let Some(str_val) = val.as_str() {
                if str_val == "" {
                    Some(Value::Null)
                } else {
                    Some(val.clone())
                }
            } else {
                Some(val.clone())
            }
        } else {
            None
        };

        BullseyeFullOption {
            ask,
            help,
            default,
            current_value: ConfigPanel::value_or_default(id, option, saved_settings).clone(),
            id: id.to_string(),
            optional: option.optional.unwrap_or(true),
            name: id.to_string(),
            option_type: option.option_type.clone(),
            pattern: None,
            fields: fields,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedAllowedEmptyOption {
    #[serde(flatten)]
    pub fields: Map<String, Value>,
    pub ask: String,
    pub id: String,
    pub name: String,
    pub optional: bool,
    #[serde(rename = "type")]
    pub option_type: String,
}

impl AppliedAllowedEmptyOption {
    pub fn from_option_with_id(
        option: &OptionToml,
        id: &String,
        container_i18n_key: Option<&String>,
    ) -> Self {
        let ask = field_i18n_single(
            "ask",
            id,
            option,
            container_i18n_key.map(|x| format!("{}_{}", x, id)),
        );

        // In full output, the OptionType may set additional defaults
        let fields = if let Ok(option_type) = OptionType::from_str(&option.option_type) {
            if let Some(type_defaults) = option_type
                .to_option_type()
                .full_extra_fields(&id, DebianRelease::Bullseye)
            {
                let mut type_defaults = type_defaults.into_iter().collect::<Map<String, Value>>();
                type_defaults.extend(option.fields.clone());
                type_defaults
            } else {
                option.fields.clone()
            }
        } else {
            option.fields.clone()
        };

        Self {
            ask,
            id: id.to_string(),
            optional: option.optional.unwrap_or(true),
            name: id.to_string(),
            option_type: option.option_type.clone(),
            fields,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybeEmptyBullseyeOption {
    NoValue(AppliedAllowedEmptyOption),
    SomeValue(BullseyeFullOption),
}
