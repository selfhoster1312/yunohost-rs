use serde_json::Value;

use std::str::FromStr;

use super::{
    field_i18n_single, field_i18n_single_optional, version::BookwormPanelVersion, ApplyAction,
    ConfigPanel, ConfigPanelError, ExcludeKey, FilterKey, Map, OptionToml, PanelToml, SectionToml,
    ALLOWED_EMPTY_TYPES,
};
use crate::helpers::{distro::DebianRelease, form::OptionType};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PanelMode {
    Bash,
    Python,
}

impl std::fmt::Display for PanelMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bash => write!(f, "{}", "bash"),
            Self::Python => write!(f, "{}", "python"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookwormFullContainer {
    pub version: BookwormPanelVersion,
    #[serde(rename = "i18n")]
    pub i18n_key: String,
    pub panels: Vec<BookwormFullPanel>,
}

impl BookwormFullContainer {
    pub fn new(i18n_key: &str) -> Self {
        Self {
            version: BookwormPanelVersion(1),
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

            let mut full_panel = BookwormFullPanel::from_panel_with_id(panel, panel_id);

            for (section_id, section) in &panel.sections {
                if !filter_key.matches_section(panel_id, section_id)
                    || exclude_key.excludes_section(panel_id, section_id)
                {
                    continue;
                }
                let mut full_section =
                    BookwormFullSection::from_section_with_id(&section, &section_id);

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
                                .push(MaybeEmptyBookwormOption::NoValue(alert_option));
                            continue;
                        }
                    }

                    let full_option = BookwormFullOption::from_option_with_id(
                        &option,
                        &option_id,
                        cp.container.i18n_key.as_ref(),
                        &saved_settings,
                    );
                    full_section
                        .options
                        .push(MaybeEmptyBookwormOption::SomeValue(full_option));
                }

                full_panel.sections.push(full_section);
            }

            full_container.panels.push(full_panel);
        }

        Ok(full_container)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookwormFullPanel {
    pub actions: ApplyAction,
    pub id: String,
    pub name: String,
    pub sections: Vec<BookwormFullSection>,
    pub services: Vec<String>,
}

impl BookwormFullPanel {
    pub fn from_panel_with_id(panel: &PanelToml, id: &str) -> BookwormFullPanel {
        BookwormFullPanel {
            actions: ApplyAction::default(),
            id: id.to_string(),
            name: panel.name.clone(),
            sections: Vec::new(),
            services: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookwormFullSection {
    pub id: String,
    #[serde(default)]
    pub is_action_section: bool,
    #[serde(default)]
    pub optional: bool,
    pub name: String,
    pub services: Vec<String>,
    pub options: Vec<MaybeEmptyBookwormOption>,
    pub visible: bool,
}

impl BookwormFullSection {
    pub fn from_section_with_id(section: &SectionToml, id: &str) -> BookwormFullSection {
        BookwormFullSection {
            id: id.to_string(),
            is_action_section: false,
            optional: section.optional.unwrap_or(true),
            name: section.name.clone(),
            services: Vec::new(),
            options: Vec::new(),
            visible: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookwormFullOption {
    #[serde(flatten)]
    /// Extra options defined by the [`OptionType`].
    // It's the first field in the struct because otherwise flattening those entries
    // will override fields that have already been set with those inside the Map.
    pub fields: Map<String, Value>,
    pub ask: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<Value>,
    pub id: String,
    // pub name: String,
    pub optional: bool,
    pub value: Value,
    #[serde(skip_serializing_if = "Self::value_none_or_null")]
    pub default: Option<Value>,
    #[serde(rename = "type")]
    pub option_type: String,
    pub visible: Value,
    pub redact: bool,
    // pattern: Option<String>,
    pub readonly: bool,
    pub mode: PanelMode,
}

impl BookwormFullOption {
    pub fn value_none_or_null(value: &Option<Value>) -> bool {
        match value {
            Some(value) => match value {
                Value::Null => true,
                _ => false,
            },
            None => true,
        }
    }

    pub fn from_option_with_id(
        option: &OptionToml,
        id: &String,
        container_i18n_key: Option<&String>,
        saved_settings: &Map<String, Value>,
    ) -> BookwormFullOption {
        let ask = field_i18n_single(
            "ask",
            id,
            option,
            container_i18n_key.map(|x| format!("{}_{}", x, id)),
        );

        let help = field_i18n_single_optional(
            "help",
            option,
            container_i18n_key.map(|x| format!("{}_{}_help", x, id)),
        )
        .map(|help_i18n| {
            if help_i18n == "" {
                Value::Object(serde_json::Map::new())
            } else {
                Value::String(help_i18n)
            }
        });

        // In full output, the OptionType may set additional defaults
        let fields = if let Ok(option_type) = OptionType::from_str(&option.option_type) {
            if let Some(type_defaults) = option_type
                .to_option_type()
                .full_extra_fields(id, DebianRelease::Bookworm)
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
        // BUT ONLY IN SETTINGS LIST NOT IN SETTINGS GET?!!!
        // So let's move this ugly logic to ConfigPanel::list... BUT WAIT WE STILL NEED A NONE TO SKIP SERIALIZATION???
        let default = if let Some("") = option.default.as_ref().map(|x| x.as_str()).flatten() {
            None
        } else {
            option.default.clone()
        };

        // let default = if let Some(val) = &option.default {
        //     if let Some(str_val) = val.as_str() {
        //         // Why is root_password{,_confirm} special here?
        //         if str_val == "" {
        //             Some(Value::Null)
        //         } else {
        //             Some(val.clone())
        //         }
        //     } else {
        //         Some(val.clone())
        //     }
        // } else {
        //     None
        // };

        // let default = if let Some(val) = &option.default {
        //     Some(val.clone())
        // } else {
        //     None
        // };

        let value = ConfigPanel::value_or_default(id, option, saved_settings).clone();
        let value = if let Ok(option_type) = OptionType::from_str(&option.option_type) {
            option_type
                .to_option_type()
                .normalize(&value)
                .unwrap_or(value)
        } else {
            value
        };

        BookwormFullOption {
            ask,
            help,
            default,
            value,
            id: id.to_string(),
            optional: option.optional.unwrap_or(true),
            // name: id.to_string(),
            option_type: option.option_type.clone(),
            // pattern: None,
            redact: fields
                .get("redact")
                .map(|x| x.as_bool())
                .flatten()
                .unwrap_or(false),
            visible: option
                .fields
                .get("visible")
                .unwrap_or(&Value::Bool(true))
                .clone(),
            fields: fields,
            readonly: false,
            mode: PanelMode::Bash,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedAllowedEmptyOption {
    #[serde(flatten)]
    pub fields: Map<String, Value>,
    pub ask: String,
    pub id: String,
    // pub name: String,
    readonly: bool,
    // pub optional: bool,
    #[serde(rename = "type")]
    pub option_type: String,
    pub visible: bool,
    pub mode: PanelMode,
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
                .full_extra_fields(&id, DebianRelease::Bookworm)
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
            // optional: option.optional.unwrap_or(true),
            // name: id.to_string(),
            option_type: option.option_type.clone(),
            fields,
            readonly: true,
            visible: true,
            mode: PanelMode::Bash,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaybeEmptyBookwormOption {
    NoValue(AppliedAllowedEmptyOption),
    SomeValue(BookwormFullOption),
}
