use snafu::prelude::*;
use toml::{Table, Value};

use std::str::FromStr;

use crate::error::*;
use crate::helpers::{file::*, form::*, i18n::*};
use crate::moulinette::i18n;

pub mod error;
mod filter_key;
pub use filter_key::FilterKey;
mod version;
pub use version::ConfigPanelVersion;

// Alias to try different maps for performance benchmark
pub type Map<K, V> = std::collections::BTreeMap<K, V>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GetMode {
    Classic,
    Export,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveMode {
    Diff,
    Full,
}

const ALLOWED_EMPTY_TYPES: &'static [OptionType] = &[
    OptionType::Alert,
    OptionType::DisplayText,
    OptionType::Markdown,
    OptionType::File,
    OptionType::Button,
];

pub struct ConfigPanel {
    entity: String,
    save_path: StrPath,
    _config_path: StrPath,
    _save_mode: SaveMode,
    // Loaded from disk
    container: ContainerToml,
}

impl ConfigPanel {
    pub fn new(
        entity: &str,
        config_path: StrPath,
        save_path: StrPath,
        save_mode: SaveMode,
    ) -> ConfigPanel {
        let container = ContainerToml::from_path(&config_path).unwrap();
        ConfigPanel {
            // TODO: we initialize empty config/values here but what about loading the config directly when Config::new()???
            entity: entity.to_string(),
            _config_path: config_path.clone(),
            save_path: save_path.clone(),
            _save_mode: save_mode,
            container,
        }
    }

    pub fn get(&self, filter: &FilterKey, mode: GetMode) -> Result<Value, Error> {
        // Logic is different based on depth of filterkey
        match filter {
            FilterKey::Option(panel, section, option) => {
                self.get_single(&panel, &section, &option, mode)
            }
            _ => self.get_multi(filter, mode),
        }
    }

    /// Get a single entry like `security.webadmin.allowlist_enable`
    pub fn get_single(
        &self,
        panel_id: &String,
        section_id: &String,
        option_id: &String,
        mode: GetMode,
    ) -> Result<Value, Error> {
        match mode {
            GetMode::Classic => {
                if let Some(option) = self
                    .container
                    .panels
                    .get(panel_id)
                    .map(|panel| {
                        panel
                            .sections
                            .get(section_id)
                            .map(|section| section.options.get(option_id))
                    })
                    .flatten()
                    .flatten()
                {
                    let saved_settings = self.saved_settings()?;

                    // TODO: error for invalid option type in config panel
                    let option_type = OptionType::from_str(&option.option_type).unwrap();
                    let value = Self::value_or_default(&option_id, &option, &saved_settings);
                    // TODO: error?

                    let value = Self::normalize(&option_type, value);
                    // println!("{:?}", value);
                    return Ok(Value::try_from(value).unwrap());
                }
            }
            _ => {
                unimplemented!("only classic mode is supported");
            }
        }

        // TODO: error when key doesn't exist
        unreachable!();
    }

    /// Get an entire panel/section, like `security` or `security.webadmin`
    ///
    /// Here the values are humanized
    pub fn get_multi(&self, filter: &FilterKey, mode: GetMode) -> Result<Value, Error> {
        // TODO: So here if it's a single value we don't do the right thing....
        let filter = filter.to_string();

        match mode {
            GetMode::Classic => {
                let classic_panel = self.to_compact().unwrap();
                Ok(Value::Table(
                    classic_panel
                        .fields
                        .into_iter()
                        .filter_map(|(x, y)| {
                            if x.starts_with(&filter) {
                                // TODO: this is a big ugly...
                                Some((x, Value::Table(Table::try_from(y).unwrap())))
                            } else {
                                None
                            }
                        })
                        .collect(),
                ))
            }
            _ => {
                unimplemented!("only classic mode is supported");
            }
        }
    }

    pub fn saved_settings(&self) -> Result<Map<String, Value>, Error> {
        let saved_settings: Map<String, Value> = if self.save_path.is_file() {
            serde_yaml_ng::from_str(&self.save_path.read().context(
                ConfigPanelReadSavePathSnafu {
                    entity: self.entity.to_string(),
                },
            )?)
            .context(ConfigPanelReadSavePathYamlSnafu {
                entity: self.entity.to_string(),
            })?
        } else {
            Map::new()
        };

        Ok(saved_settings)
    }

    /// The values are normalized/humanized for use in get_multi
    pub fn to_compact(&self) -> Result<AppliedCompactContainer, Error> {
        let saved_settings = self.saved_settings()?;

        let mut compact_container = AppliedCompactContainer::new();

        for (panel_id, panel) in &self.container.panels {
            for (section_id, section) in &panel.sections {
                for (option_id, option) in &section.options {
                    // Maybe we should skip this option because it doesn't have an actual value?
                    // if let Some(bind) = option.get("bind").map(|x| x.as_str()).flatten() {
                    //     // TODO: what is this?
                    //     continue;
                    // }

                    let ask = self.ask_i18n(&option_id, &option);

                    if let Ok(option_type) = OptionType::from_str(&option.option_type) {
                        // Apparently at least for alert we have the ask to insert, but no value...
                        // TODO: Is that true for other ALLOWED_EMPTY_TYPES?
                        if ALLOWED_EMPTY_TYPES.contains(&option_type) {
                            compact_container.fields.insert(
                                format!("{}.{}.{}", panel_id, section_id, option_id),
                                AppliedCompactValue::new(ask, None),
                            );
                            continue;
                        }
                    }

                    let value = Self::value_or_default(&option_id, &option, &saved_settings);
                    // TODO: error for invalid option type in config panel
                    let option_type = OptionType::from_str(&option.option_type).unwrap();
                    let value = Self::humanize(&option_type, value);
                    // println!("normalized: {:?}", value);
                    compact_container.fields.insert(
                        format!("{}.{}.{}", panel_id, section_id, option_id),
                        AppliedCompactValue::new(ask, Some(value)),
                    );
                }
            }
        }

        Ok(compact_container)
    }

    pub fn value_or_default<'a>(
        option_id: &'a String,
        option: &'a OptionToml,
        saved_settings: &'a Map<String, Value>,
    ) -> &'a Value {
        // In the saved settings, the value is saved with the option id without the parent section/panel path...
        // UNWRAP NOTE: Normally, we have previously skipped entries whose type don't have a default value
        saved_settings
            .get(option_id)
            .unwrap_or(&option.default.as_ref().unwrap())
    }

    pub fn humanize(option_type: &OptionType, val: &Value) -> String {
        let option_type = option_type.to_option_type();

        // Omit passwords
        if option_type.hide_user_input_in_prompt() {
            return "**************".to_string();
        }

        // Some option types don't have to do normalization, in which case it's None
        if let Some(humanized) = option_type.humanize(val) {
            return humanized.to_string();
        }

        if let Some(stringy_value) = val.as_str() {
            // Don't escape the string stuff like quotes if it's already a string
            stringy_value.to_string()
        } else {
            val.to_string()
        }
    }

    pub fn normalize(option_type: &OptionType, val: &Value) -> Value {
        let option_type = option_type.to_option_type();

        // Some option types don't have to do normalization, in which case it's None
        if let Some(normalized) = option_type.normalize(val) {
            normalized
        } else {
            val.clone()
        }
    }

    pub fn ask_i18n(&self, option_id: &str, option: &OptionToml) -> String {
        let mut ask = None;
        if let Some(option_ask_table) = option.fields.get("ask").map(|x| x.as_table()).flatten() {
            // If the ask field is set, it's always a table containing different translations for this setting
            // See docs about _value_for_locale. In that case, we want to select the suitable translation,
            // or the first one that comes.
            ask = Some(_value_for_locale(option_ask_table));
        } else if let Some(i18n_key) = &self.container.i18n_key {
            let option_i18n_key = format!("{}_{}", i18n_key, option_id);
            // TODO: error
            ask = Some(i18n::yunohost_no_context(&option_i18n_key).unwrap());
        }

        // TODO: Is this always true?
        let ask = ask.expect(&format!(
            "Woops, ask was empty for option id {:?}",
            option_id
        ));

        return ask;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Translation {
    Str(String),
    I18N(Map<String, String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContainerToml {
    version: ConfigPanelVersion,
    #[serde(rename = "i18n")]
    i18n_key: Option<String>,
    #[serde(flatten)]
    pub panels: Map<String, PanelToml>,
}

impl ContainerToml {
    pub fn from_path(path: &StrPath) -> Result<Self, Error> {
        // TODO: error management
        Ok(toml::from_str(&path.read().unwrap()).unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PanelToml {
    name: String,
    #[serde(flatten)]
    pub sections: Map<String, SectionToml>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionToml {
    name: String,
    #[serde(flatten)]
    pub options: Map<String, OptionToml>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptionToml {
    #[serde(rename = "type")]
    option_type: String,
    default: Option<Value>,
    #[serde(flatten)]
    pub fields: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedCompactContainer {
    #[serde(flatten)]
    pub fields: Map<String, AppliedCompactValue>,
}

impl AppliedCompactContainer {
    pub fn new() -> Self {
        Self { fields: Map::new() }
    }
}

/// Once we have applied settings and translated stuff, only ask/value remain in the compact view.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppliedCompactValue {
    pub ask: String,
    // TODO: why is everything always a string???
    // Actually, for type="alert", we have a "ask" but no value
    pub value: Option<String>,
}

impl AppliedCompactValue {
    pub fn new(ask: String, value: Option<String>) -> Self {
        Self { ask, value }
    }
}
