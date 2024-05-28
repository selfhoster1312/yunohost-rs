use serde_json::Value;

use std::str::FromStr;

use super::{
    field_i18n_single, field_i18n_single_optional_bullseye_englishname, ConfigPanel,
    ConfigPanelVersion, Map, OptionToml, PanelToml, SectionToml,
};
use crate::helpers::form::OptionType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedFullContainer {
    pub version: ConfigPanelVersion,
    #[serde(rename = "i18n")]
    pub i18n_key: String,
    pub panels: Vec<AppliedFullPanel>,
}

impl AppliedFullContainer {
    pub fn new(i18n_key: &str) -> Self {
        Self {
            version: ConfigPanelVersion::V1_0,
            i18n_key: i18n_key.to_string(),
            panels: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedFullPanel {
    pub actions: ApplyAction,
    pub id: String,
    pub name: EnglishName,
    pub sections: Vec<AppliedFullSection>,
    pub services: Vec<String>,
}

impl AppliedFullPanel {
    pub fn from_panel_with_id(panel: &PanelToml, id: &str) -> AppliedFullPanel {
        AppliedFullPanel {
            actions: ApplyAction::default(),
            id: id.to_string(),
            name: EnglishName::new(&panel.name),
            sections: Vec::new(),
            services: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedFullSection {
    pub id: String,
    #[serde(default)]
    pub is_action_section: bool,
    #[serde(default)]
    pub optional: bool,
    pub name: EnglishName,
    pub services: Vec<String>,
    pub options: Vec<MaybeEmptyOption>,
}

impl AppliedFullSection {
    pub fn from_section_with_id(section: &SectionToml, id: &str) -> AppliedFullSection {
        AppliedFullSection {
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
pub struct AppliedFullOption {
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

impl AppliedFullOption {
    pub fn from_option_with_id(
        option: &OptionToml,
        id: &String,
        container_i18n_key: Option<&String>,
        saved_settings: &Map<String, Value>,
    ) -> AppliedFullOption {
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
            if let Some(type_defaults) = option_type.to_option_type().full_extra_fields(id) {
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

        AppliedFullOption {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ApplyAction {
    apply: ApplyEnglishAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplyEnglishAction {
    en: String,
}

impl Default for ApplyEnglishAction {
    fn default() -> Self {
        Self {
            en: "Apply".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnglishName {
    en: String,
}

impl EnglishName {
    pub fn new(name: &str) -> Self {
        Self {
            en: name.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedAllowedEmptyOption {
    pub ask: String,
    pub id: String,
    pub name: String,
    pub optional: bool,
    #[serde(rename = "type")]
    pub option_type: String,
    #[serde(flatten)]
    pub fields: Map<String, Value>,
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
            if let Some(type_defaults) = option_type.to_option_type().full_extra_fields(&id) {
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
pub enum MaybeEmptyOption {
    NoValue(AppliedAllowedEmptyOption),
    SomeValue(AppliedFullOption),
}
