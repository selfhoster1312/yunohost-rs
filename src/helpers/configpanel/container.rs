use serde::{Deserialize, Serialize};
use toml::{Table, Value};

use crate::{helpers::i18n::_value_for_locale, moulinette::i18n};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContainerModel {
    id: String,
    name: Option<String>,
    services: Vec<String>,
    help: Option<String>,
    #[serde(flatten)]
    attrs: Table,
}

impl ContainerModel {
    /// Translate `ask` and `name` attributes of panels and section.
    /// This is in-place mutation.
    pub fn translate(&mut self, i18n_key: Option<String>) {
        for key in ["help".to_string(), "name".to_string()] {
            if let Some(current_value) = self.attrs.get_mut(&key) {
                *current_value =
                    Value::String(_value_for_locale(&current_value.as_table().unwrap()));
            } else {
                match (key.as_str(), &i18n_key) {
                    ("help", Some(i18n_key)) => {
                        let i18n_key = format!("{i18n_key}.{}_help", &self.id);
                        // UNWRAP NOTE: Init can fail but otherwise this is safe..
                        if i18n::n_exists(&i18n_key).unwrap() {
                            // UNWRAP NOTE: We just checked if the key exists so this is safe
                            self.attrs
                                .insert(key, Value::String(i18n::n(&i18n_key, None).unwrap()));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
