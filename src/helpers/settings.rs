use serde_json::Value;

use std::str::FromStr;

use crate::helpers::{
    configpanel::{error::ConfigPanelError, ConfigPanel, FilterKey, GetMode, SaveMode},
    legacy::*,
};

pub struct SettingsConfigPanel {
    panel: ConfigPanel,
    _virtual_settings: Vec<&'static str>,
}

impl SettingsConfigPanel {
    pub fn new() -> Result<SettingsConfigPanel, ConfigPanelError> {
        Ok(SettingsConfigPanel {
            panel: ConfigPanel::new(
                "settings",
                "/usr/share/yunohost/config_global.toml".into(),
                "/etc/yunohost/settings.yml".into(),
                SaveMode::Diff,
            )?,
            _virtual_settings: vec![
                "root_password",
                "root_password_confirm",
                "passwordless_sudo",
            ],
        })
    }

    pub fn get(
        &mut self,
        key: &SettingsFilterKey,
        mode: GetMode,
    ) -> Result<Value, ConfigPanelError> {
        let key: FilterKey = key.clone().into();
        let result = self.panel.get(&key, mode)?;

        if let Some(result_str) = result.as_str() {
            if result_str == "True" {
                return Ok(Value::Bool(true));
            } else if result_str == "False" {
                return Ok(Value::Bool(false));
            }
        }

        return Ok(result);
    }
}

/// This is a special [`FilterKey`] where legacy settings key are supported.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsFilterKey {
    Panel(String),
    Section(String, String),
    Option(String, String, String),
}

impl std::fmt::Display for SettingsFilterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Panel(p) => write!(f, "{}", p),
            Self::Section(p, s) => write!(f, "{}.{}", p, s),
            Self::Option(p, s, o) => write!(f, "{}.{}.{}", p, s, o),
        }
    }
}

impl FromStr for SettingsFilterKey {
    type Err = ConfigPanelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "" {
            return Err(ConfigPanelError::FilterKeyNone);
        }

        // Translate from legacy settings
        let s = translate_legacy_settings_to_configpanel_settings(s);

        let mut parts = s.split('.');

        let panel = parts.next().unwrap().to_string();

        match parts.next() {
            None => Ok(Self::Panel(panel)),
            Some(section) => {
                let section = section.to_string();
                match parts.next() {
                    None => Ok(Self::Section(panel, section)),
                    Some(option) => {
                        if parts.next().is_some() {
                            return Err(ConfigPanelError::FilterKeyTooDeep {
                                filter_key: s.to_string(),
                            });
                        }

                        let option = option.to_string();
                        Ok(Self::Option(panel, section, option))
                    }
                }
            }
        }
    }
}

impl From<SettingsFilterKey> for FilterKey {
    fn from(sfk: SettingsFilterKey) -> FilterKey {
        match sfk {
            SettingsFilterKey::Panel(p) => FilterKey::Panel(p),
            SettingsFilterKey::Section(p, s) => FilterKey::Section(p, s),
            SettingsFilterKey::Option(p, s, o) => FilterKey::Option(p, s, o),
        }
    }
}
