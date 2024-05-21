// use snafu::prelude::*;

use crate::error::*;
use crate::helpers::configpanel::*;
use toml::Value;

pub struct SettingsConfigPanel {
    panel: ConfigPanel,
    _virtual_settings: Vec<&'static str>,
}

impl SettingsConfigPanel {
    pub fn new() -> SettingsConfigPanel {
        SettingsConfigPanel {
            panel: ConfigPanel::new(
                "settings",
                "/usr/share/yunohost/config_global.toml".into(),
                "/etc/yunohost/settings.yml".into(),
                SaveMode::Diff,
            ),
            _virtual_settings: vec![
                "root_password",
                "root_password_confirm",
                "passwordless_sudo",
            ],
        }
    }

    pub fn get(&mut self, key: &str, mode: GetMode) -> Result<Value, Error> {
        let result = self.panel.get(key, mode)?;

        match mode {
            GetMode::Full => {
                // TODO: add i18n help
                unimplemented!("oh noes i18n");
            }
            _ => {
                if let Some(result_str) = result.as_str() {
                    if result_str == "True" {
                        return Ok(Value::Boolean(true));
                    } else if result_str == "False" {
                        return Ok(Value::Boolean(false));
                    }
                }

                return Ok(result);
            }
        }
    }
}
