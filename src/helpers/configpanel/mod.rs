use serde_json::Value;
use snafu::prelude::*;

use std::str::FromStr;

// use crate::helpers::{file::*, form::*, i18n::*};
use crate::helpers::{
    distro::{debian_version, DebianRelease},
    file::*,
    form::*,
    i18n,
};

// Different GetMode
mod classic;
mod export_bookworm;
mod export_bullseye;
mod full_bookworm;
mod full_bullseye;

pub mod error;
use error::*;
mod exclude_key;
pub use exclude_key::ExcludeKey;
mod filter_key;
pub use filter_key::FilterKey;
mod version;
pub use version::BullseyePanelVersion;

// Alias to try different maps for performance benchmark
pub(crate) type Map<K, V> = std::collections::BTreeMap<K, V>;

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
    ) -> Result<ConfigPanel, ConfigPanelError> {
        // Load the ConfigPanel configuration, eg. /usr/share/yunohost/config_global.toml
        let container: ContainerToml =
            config_path
                .read_toml()
                .context(ConfigPanelConfigReadSnafu {
                    entity: entity.to_string(),
                    path: config_path.clone(),
                })?;

        Ok(ConfigPanel {
            entity: entity.to_string(),
            _config_path: config_path.clone(),
            save_path: save_path.clone(),
            _save_mode: save_mode,
            container,
        })
    }

    pub fn get(&self, filter_key: &FilterKey, mode: GetMode) -> Result<Value, ConfigPanelError> {
        match filter_key {
            FilterKey::Option(panel_id, section_id, option_id) => {
                self.get_single(filter_key, &panel_id, &section_id, &option_id, mode)
            }
            _ => self.get_multi(filter_key, mode, &ExcludeKey::Nothing),
        }
    }

    pub fn list(&self, mode: GetMode) -> Result<Value, ConfigPanelError> {
        // Seriously, different ExcludeKey based on GetMode?
        let exclude = match mode {
            GetMode::Classic => {
                // filter security.root_access... WHY?
                ExcludeKey::Section("security".to_string(), "root_access".to_string())
            }
            _ => ExcludeKey::Nothing,
        };

        self.get_multi(&FilterKey::Everything, mode, &exclude)
    }

    /// Get a single entry like `security.webadmin.allowlist_enable`
    pub fn get_single(
        &self,
        filter_key: &FilterKey,
        panel_id: &String,
        section_id: &String,
        option_id: &String,
        mode: GetMode,
    ) -> Result<Value, ConfigPanelError> {
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

                    let option_type = OptionType::from_str(&option.option_type).context(
                        OptionTypeWrongSnafu {
                            option_id: option_id.to_string(),
                            option_type: option.option_type.to_string(),
                        },
                    )?;
                    let value = Self::value_or_default(&option_id, &option, &saved_settings);

                    let value = Self::normalize(&option_type, value);
                    // TODO: is this always ok to unwrap?
                    return Ok(Value::try_from(value).unwrap());
                }
            }
            GetMode::Export => {
                let value = self.get_multi(&filter_key, mode, &ExcludeKey::Nothing)?;
                return Ok(serde_json::to_value(value).unwrap());
            }
            GetMode::Full => {
                let value = self.get_multi(&filter_key, mode, &ExcludeKey::Nothing)?;
                // TODO: is this always ok to unwrap?
                return Ok(serde_json::to_value(value).unwrap());
            }
        }

        // The requested FilterKey was not found in the ConfigPanel, return an error
        return Err(ConfigPanelError::FilterKeyNotFound {
            entity: self.entity.to_string(),
            filter_key: filter_key.clone(),
        });
    }

    /// Get an entire panel/section, like `security` or `security.webadmin`
    ///
    /// Here the values are humanized
    pub fn get_multi(
        &self,
        filter: &FilterKey,
        mode: GetMode,
        exclude_key: &ExcludeKey,
    ) -> Result<Value, ConfigPanelError> {
        match mode {
            GetMode::Classic => {
                let classic_panel =
                    classic::AppliedClassicContainer::from_config_panel(self, filter, exclude_key)?;
                Ok(serde_json::to_value(classic_panel).unwrap())
            }
            GetMode::Export => match debian_version().unwrap() {
                DebianRelease::Bullseye => {
                    let export_panel = export_bullseye::ExportBullseyeContainer::from_config_panel(
                        self,
                        filter,
                        exclude_key,
                    )?;
                    Ok(serde_json::to_value(export_panel).unwrap())
                }
                DebianRelease::Bookworm => {
                    let export_panel = export_bookworm::ExportBookwormContainer::from_config_panel(
                        self,
                        filter,
                        exclude_key,
                    )?;
                    Ok(serde_json::to_value(export_panel).unwrap())
                }
            },
            GetMode::Full => {
                // So depending on Debian version we do something different...
                // TODO: what do we do when running tests? Do we have global state that the test runner can override?
                match debian_version().unwrap() {
                    DebianRelease::Bullseye => {
                        let full_panel = full_bullseye::BullseyeFullContainer::from_config_panel(
                            &self,
                            filter,
                            exclude_key,
                        )?;
                        Ok(serde_json::to_value(full_panel).unwrap())
                    }
                    DebianRelease::Bookworm => {
                        let mut full_panel =
                            full_bookworm::BookwormFullContainer::from_config_panel(
                                &self,
                                filter,
                                exclude_key,
                            )?;

                        // WAIT WTF IS THIS SHIT
                        // BELOW IS UGLY SHIT! Because apparently in list mode passwords need to be replaced with "" instead of null
                        // Or no wait is it all password type fields?
                        for panel in &mut full_panel.panels {
                            if panel.id != "security" {
                                continue;
                            }

                            for section in &mut panel.sections {
                                if section.id != "root_access" {
                                    continue;
                                }

                                for option in &mut section.options {
                                    match option {
                                        full_bookworm::MaybeEmptyBookwormOption::SomeValue(
                                            ref mut option,
                                        ) => {
                                            if option.option_type == "password" {
                                                option.value = Value::String("".to_string());
                                            }
                                        }
                                        _ => continue,
                                    }
                                }
                            }
                        }

                        Ok(serde_json::to_value(full_panel).unwrap())
                    }
                }
            }
        }
    }

    pub fn saved_settings(&self) -> Result<Map<String, Value>, ConfigPanelError> {
        let saved_settings: Map<String, Value> = if self.save_path.is_file() {
            self.save_path
                .read_yaml()
                .context(ConfigPanelSaveReadSnafu {
                    entity: self.entity.to_string(),
                    path: self.save_path.clone(),
                })?
        } else {
            Map::new()
        };

        Ok(saved_settings)
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
            .unwrap_or(&option.default.as_ref().expect(&format!(
                "OptionID {option_id} does not have default value, and does not have saved setting"
            )))
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContainerToml {
    version: BullseyePanelVersion,
    #[serde(rename = "i18n")]
    i18n_key: Option<String>,
    #[serde(flatten)]
    pub panels: Map<String, PanelToml>,
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
    optional: Option<bool>,
    #[serde(flatten)]
    pub options: Map<String, OptionToml>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptionToml {
    optional: Option<bool>,
    #[serde(rename = "type")]
    option_type: String,
    default: Option<Value>,
    #[serde(flatten)]
    pub fields: Map<String, Value>,
}

/// Translate specific option field to the current locale.
///
/// First lookup if field `field` contains a table. For example:
///
/// ```
/// ask:
///   en: The question?
///   fr: The question?
/// ```
///
/// If so, use the current locale if possible, see [`_value_for_locale`](i18n::_value_for_locale).
///
/// Otherwise, lookup in Yunohost translations the key `CONTAINERID_OPTIONID`. For example,
/// `global_settings_setting_nginx_compatibility_help`.
///
/// Otherwise, return the field value as is.
///
/// This is used for `help` field in full view. Returns None if none of those options is available.
pub fn field_i18n_single_optional(
    field: &str,
    option: &OptionToml,
    i18n_key: Option<String>,
) -> Option<String> {
    // if let Some(option_field_table) = option.fields.get(field).map(|x| x.as_table()).flatten() {
    if let Some(option_field_table) = option.fields.get(field).map(|x| x.as_object()).flatten() {
        // If the ask field is set, it's always a table containing different translations for this setting
        // See docs about _value_for_locale. In that case, we want to select the suitable translation,
        // or the first one that comes.
        return Some(i18n::_value_for_locale(option_field_table));
    } else if let Some(i18n_key) = i18n_key {
        // If the translation key exists in the locale, use it... otherwise don't touch the ask field
        if let Ok(translation) = i18n::yunohost_no_context(&i18n_key) {
            return Some(translation);
        }
    }

    return option
        .fields
        .get(field)
        .map(|x| x.as_str())
        .flatten()
        .map(|x| x.to_string());
}

/// Translate specific option field to the current locale.
///
/// First lookup if field `field` contains a table. For example:
///
/// ```
/// ask:
///   en: The question?
///   fr: The question?
/// ```
///
/// If so, use the current locale if possible, see [`_value_for_locale`](i18n::_value_for_locale).
///
/// Otherwise, lookup in Yunohost translations the key `CONTAINERID_OPTIONID`. For example,
/// `global_settings_setting_nginx_compatibility_help`.
///
/// Otherwise, return the field value as is.
///
/// This is used for `ask` field in classic/full view.
///
/// Panics if none of these options is available.
pub fn field_i18n_single(
    field: &str,
    option_id: &str,
    option: &OptionToml,
    i18n_key: Option<String>,
) -> String {
    field_i18n_single_optional(field, option, i18n_key).expect(&format!(
        "Woops, field {field} was empty (or non-str) for option id {option_id}"
    ))
}

// TODO: should this even exist in bookworm?
pub fn field_i18n_single_optional_bullseye_englishname(
    field: &str,
    option: &OptionToml,
    i18n_key: Option<String>,
) -> Option<Value> {
    // if let Some(option_field_table) = option.fields.get(field).map(|x| x.as_table()).flatten() {
    if let Some(option_field_table) = option.fields.get(field).map(|x| x.as_object()).flatten() {
        // If the ask field is set, it's always a table containing different translations for this setting
        // See docs about _value_for_locale. In that case, we want to select the suitable translation,
        // or the first one that comes.
        return Some(Value::String(i18n::_value_for_locale(option_field_table)));
    } else if let Some(i18n_key) = i18n_key {
        // If the translation key exists in the locale, use it... otherwise don't touch the ask field
        if let Ok(translation) = i18n::yunohost_no_context(&i18n_key) {
            return Some(Value::String(translation));
        }
    }

    return option
        .fields
        .get(field)
        .map(|x| x.as_str())
        .flatten()
        .map(|x| {
            // THIS IS THE WTF PART
            if x == "" {
                // UNWRAP NOTE: safe unwrap
                serde_json::to_value(EnglishName::new("")).unwrap()
            } else {
                Value::String(x.to_string())
            }
        });
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
