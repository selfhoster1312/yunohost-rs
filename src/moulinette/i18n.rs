use camino::Utf8Path;
use snafu::prelude::*;

use std::env;
use std::sync::{OnceLock, RwLock};

use crate::error::*;
use crate::helpers::file::*;

use std::collections::HashMap;

pub static DEFAULT_LOCALE_FALLBACK: &'static str = "en";

// Wrap in OnceLock to make sure it's only initialized once. Wrap in RwLock to allow inner mutability.
pub static STATIC_I18N: OnceLock<RwLock<Moulinette18n>> = OnceLock::new();

pub struct Translator {
    // locale_dir: Utf8PathBuf,
    locale: String,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Translator {
    pub fn new(locale_dir: &Utf8Path, default_locale: &str) -> Result<Translator, Error> {
        let mut translations: HashMap<String, HashMap<String, String>> = HashMap::new();
        let read_dir = ReadDir::new(&locale_dir).context(LocalesReadFailedSnafu {
            path: locale_dir.to_path_buf(),
        })?;
        for locale in &read_dir.paths() {
            // UNWRAP NOTE: Safe unwrap because it's called with entries from read_dir, which cannot be ".."
            let file_name = locale.file_name().unwrap();

            if !file_name.ends_with(".json") {
                continue;
            }

            let locale_name = file_name.trim_end_matches(".json");

            let locale_str = read(&locale).context(LocalesReadFailedSnafu {
                path: locale.to_path_buf(),
            })?;
            let locale_values: HashMap<String, String> = serde_json::from_str(&locale_str)
                .context(LocalesLoadFailedSnafu {
                    path: locale.to_path_buf(),
                })?;

            translations.insert(locale_name.to_string(), locale_values);
        }

        Ok(Translator {
            locale: default_locale.to_string(),
            translations,
        })
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    pub fn key_exists(&self, key: &str) -> bool {
        self.translations
            .get(&"en".to_string())
            .unwrap()
            .contains_key(key)
    }

    pub fn translate(
        &self,
        key: &str,
        context: Option<HashMap<String, String>>,
    ) -> Result<String, Error> {
        // UNWRAP NOTE: If the language requested doesn't exist, this unwrap is the least of your worries
        // TODO: maybe check the language exists when we change it?!
        let raw_translation =
            if let Some(val) = self.translations.get(&self.locale).unwrap().get(key) {
                val.to_string()
            } else if let Some(val) = self
                .translations
                .get(DEFAULT_LOCALE_FALLBACK)
                .unwrap()
                .get(key)
            {
                val.to_string()
            } else {
                return Err(Error::LocalesMissingKey {
                    key: key.to_string(),
                });
            };

        if let Some(context) = context {
            strfmt::strfmt(&raw_translation, &context).context(LocalesFormattingSnafu {
                key: raw_translation.to_string(),
                args: context.clone(),
            })
        } else {
            Ok(raw_translation)
        }
    }
}

pub struct Moulinette18n {
    pub current_locale: String,
    pub yunohost_translator: Translator,
    pub moulinette_translator: Translator,
}

impl Moulinette18n {
    pub fn new(locale: &str) -> Result<Moulinette18n, Error> {
        let yunohost_translator = Translator::new("/usr/share/yunohost/locales".into(), locale)
            .context(Moulinette18nYunohostSnafu)?;
        let moulinette_translator = Translator::new("/usr/share/moulinette/locales".into(), locale)
            .context(Moulinette18nMoulinetteSnafu)?;
        Ok(Moulinette18n {
            yunohost_translator,
            moulinette_translator,
            current_locale: locale.to_string(),
        })
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.current_locale = locale.to_string();
        self.yunohost_translator.set_locale(locale);
        self.moulinette_translator.set_locale(locale);
    }

    pub fn get_locale(&self) -> String {
        self.current_locale.to_string()
    }
}

pub fn get_system_locale() -> String {
    let locale = env::var("LC_ALL").or_else(|_| env::var("LANG")).unwrap();

    locale.chars().take(2).collect()
}

pub fn init() -> Result<&'static RwLock<Moulinette18n>, Error> {
    let locale = get_system_locale();

    if let Some(moulinette18n) = STATIC_I18N.get() {
        Ok(moulinette18n)
    } else {
        let moulinette18n = Moulinette18n::new(&locale)?;
        Ok(STATIC_I18N.get_or_init(|| RwLock::new(moulinette18n)))
    }
}

pub fn g(key: &str, context: Option<HashMap<String, String>>) -> Result<String, Error> {
    let i18n = if let Some(i18n) = STATIC_I18N.get() {
        i18n.read().unwrap()
    } else {
        init()?.read().unwrap()
    };

    i18n.moulinette_translator.translate(key, context)
}

pub fn n(key: &str, context: Option<HashMap<String, String>>) -> Result<String, Error> {
    let i18n = if let Some(i18n) = STATIC_I18N.get() {
        i18n.read().unwrap()
    } else {
        init()?.read().unwrap()
    };

    i18n.yunohost_translator.translate(key, context)
}

pub fn n_exists(key: &str) -> Result<bool, Error> {
    let i18n = if let Some(i18n) = STATIC_I18N.get() {
        i18n.read().unwrap()
    } else {
        init()?.read().unwrap()
    };

    Ok(i18n.yunohost_translator.key_exists(key))
}

pub fn get_locale() -> String {
    STATIC_I18N.get().unwrap().read().unwrap().get_locale()
}

pub fn set_locale(locale: &str) {
    let mut m18n = STATIC_I18N.get().unwrap().write().unwrap();
    m18n.set_locale(locale);
}
