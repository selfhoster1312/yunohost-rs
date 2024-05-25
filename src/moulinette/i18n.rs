//! # Moulinette18n
//!
//! This module contains helpers for internationalization. Yunohost translations are contained in one JSON file per locale,
//! in the /usr/share/{yunohost,moulinette}/locales/ directories.
//!
//! There are separate methods for accessing `yunohost` and `moulinette` translations, and they are prefixed accordingly.
//!
//! Contrary to the Python version, you do not need to explicitly initialize the translation system before you start asking for
//! translations. The translations will be loaded (just once) the first time you request them.

use camino::Utf8Path;
use snafu::prelude::*;

use std::env;
// ASYNC TODO: Replace std::sync::RwLock with tokio::sync::RwLock if we ever go async
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::error::*;
use crate::helpers::file::*;

use std::collections::HashMap;

pub(crate) static DEFAULT_LOCALE_FALLBACK: &'static str = "en";

// Wrap in OnceLock to make sure it's only initialized once. Wrap in RwLock to allow inner mutability (Translator::set_locale)
pub(crate) static YUNOHOST_GLOBAL_I18N: OnceLock<RwLock<Translator>> = OnceLock::new();
pub(crate) static YUNOHOST_LOCALES_DIR: &'static str = "/usr/share/yunohost/locales";

pub(crate) static MOULINETTE_GLOBAL_I18N: OnceLock<RwLock<Translator>> = OnceLock::new();
pub(crate) static MOULINETTE_LOCALES_DIR: &'static str = "/usr/share/moulinette/locales";

type Translation = HashMap<String, String>;

enum MaybeTranslation {
    NotLoaded(camino::Utf8PathBuf),
    Loaded(Translation),
}

pub(crate) struct Translator {
    locale: String,
    translations: HashMap<String, RwLock<MaybeTranslation>>,
}

impl Translator {
    pub(crate) fn init(
        state: &'static OnceLock<RwLock<Translator>>,
        locales_dir: &Utf8Path,
    ) -> Result<&'static RwLock<Translator>, Error> {
        let locale = get_system_locale();

        if let Some(translator) = state.get() {
            Ok(translator)
        } else {
            let translator = Translator::new(locales_dir, &locale)?;
            Ok(state.get_or_init(|| RwLock::new(translator)))
        }
    }

    pub fn new(locales_dir: &Utf8Path, default_locale: &str) -> Result<Translator, Error> {
        let mut translations = HashMap::new();
        let read_dir = ReadDir::new(&locales_dir).context(LocalesReadFailedSnafu {
            path: locales_dir.to_path_buf(),
        })?;
        for locale in &read_dir.paths() {
            // UNWRAP NOTE: Safe unwrap because it's called with entries from read_dir, which cannot be ".."
            let file_name = locale.file_name().unwrap();

            if !file_name.ends_with(".json") {
                continue;
            }

            let locale_name = file_name.trim_end_matches(".json");

            if locale_name == "en"
                || locale_name == default_locale
                || locale_name == DEFAULT_LOCALE_FALLBACK
            {
                let translation = Self::load_locale(locale)?;
                translations.insert(
                    locale_name.to_string(),
                    RwLock::new(MaybeTranslation::Loaded(translation)),
                );
            } else {
                translations.insert(
                    locale_name.to_string(),
                    RwLock::new(MaybeTranslation::NotLoaded(locale.clone())),
                );
            }
        }

        Ok(Translator {
            locale: default_locale.to_string(),
            translations,
        })
    }

    fn load_locale(locale: &Utf8Path) -> Result<Translation, Error> {
        let locale = StrPath::from(locale);
        let locale_str = locale.read().context(LocalesReadFailedSnafu {
            path: locale.to_path_buf(),
        })?;
        let locale_values = serde_json::from_str(&locale_str).context(LocalesLoadFailedSnafu {
            path: locale.to_path_buf(),
        })?;
        Ok(locale_values)
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    pub fn get_locale(&self) -> String {
        self.locale.to_string()
    }

    pub fn key_exists(&self, key: &str) -> bool {
        let locale = self
            .translations
            .get(&"en".to_string())
            .unwrap()
            .read()
            .unwrap();
        match &*locale {
            MaybeTranslation::Loaded(translation) => translation.contains_key(key),
            _ => false,
        }
    }

    pub fn translate_no_context(&self, key: &str) -> Result<String, Error> {
        // UNWRAP NOTE: If the language requested doesn't exist, this unwrap is the least of your worries
        // TODO: maybe check the language exists when we change it?!
        if let Some(locale) = self.translations.get(&self.locale) {
            {
                let mut locale_mut = locale.write().unwrap();
                if let MaybeTranslation::NotLoaded(locale) = &*locale_mut {
                    let locale = Self::load_locale(&locale)?;
                    *locale_mut = MaybeTranslation::Loaded(locale);
                }
            }
            let locale = locale.read().unwrap();
            let locale = match &*locale {
                MaybeTranslation::Loaded(locale) => locale,
                _ => unreachable!(),
            };
            if let Some(val) = locale.get(key) {
                return Ok(val.to_string());
            }
        }
        let locale = self
            .translations
            .get(DEFAULT_LOCALE_FALLBACK)
            .unwrap()
            .read()
            .unwrap();
        let locale = match &*locale {
            MaybeTranslation::Loaded(locale) => locale,
            _ => unreachable!(),
        };
        let val = locale.get(key).context(LocalesMissingKeySnafu {
            key: key.to_string(),
        })?;
        Ok(val.to_string())
    }

    pub fn translate_with_context(
        &self,
        key: &str,
        context: HashMap<String, String>,
    ) -> Result<String, Error> {
        let raw_translation = self.translate_no_context(key)?;

        strfmt::strfmt(&raw_translation, &context).context(LocalesFormattingSnafu {
            key: raw_translation.to_string(),
            args: context.clone(),
        })
    }
}

pub(crate) fn get_system_locale() -> String {
    let locale = env::var("LC_ALL").or_else(|_| env::var("LANG")).unwrap();

    locale.chars().take(2).collect()
}

pub(crate) fn moulinette_load() -> Result<RwLockReadGuard<'static, Translator>, Error> {
    if let Some(translator) = MOULINETTE_GLOBAL_I18N.get() {
        Ok(translator.read().unwrap())
    } else {
        Ok(
            Translator::init(&MOULINETTE_GLOBAL_I18N, MOULINETTE_LOCALES_DIR.into())?
                .read()
                .unwrap(),
        )
    }
}

pub(crate) fn moulinette_load_mut() -> Result<RwLockWriteGuard<'static, Translator>, Error> {
    if let Some(translator) = MOULINETTE_GLOBAL_I18N.get() {
        Ok(translator.write().unwrap())
    } else {
        // Ok(moulinette_init()?.write().unwrap())
        Ok(
            Translator::init(&MOULINETTE_GLOBAL_I18N, MOULINETTE_LOCALES_DIR.into())?
                .write()
                .unwrap(),
        )
    }
}

/// Gets a Moulinette translation with a given context.
///
/// This method substitutes key names in the context HashMap
/// with the corresponding values in the formatted translation.
///
/// For example, for a translation key `user_not_found` containing the translation string:
/// `User was not found: {username}`, `moulinette_context("user_not_found", hashmap!{"username": "toto"})`
/// will produce the string `User was not found: toto`.
pub fn moulinette_context(key: &str, context: HashMap<String, String>) -> Result<String, Error> {
    let translator = moulinette_load()?;
    translator.translate_with_context(key, context)
}

/// Gets a Moulinette translation without context (variable substitution)
pub fn moulinette_no_context(key: &str) -> Result<String, Error> {
    let translator = moulinette_load()?;
    translator.translate_no_context(key)
}

/// Checks whether a translation key exists in Moulinette strings
pub fn moulinette_exists(key: &str) -> Result<bool, Error> {
    let translator = moulinette_load()?;
    Ok(translator.key_exists(key))
}

pub(crate) fn yunohost_load() -> Result<RwLockReadGuard<'static, Translator>, Error> {
    if let Some(translator) = YUNOHOST_GLOBAL_I18N.get() {
        Ok(translator.read().unwrap())
    } else {
        Ok(
            Translator::init(&YUNOHOST_GLOBAL_I18N, YUNOHOST_LOCALES_DIR.into())?
                .read()
                .unwrap(),
        )
    }
}

pub(crate) fn yunohost_load_mut() -> Result<RwLockWriteGuard<'static, Translator>, Error> {
    if let Some(translator) = YUNOHOST_GLOBAL_I18N.get() {
        Ok(translator.write().unwrap())
    } else {
        // Ok(yunohost_init()?.write().unwrap())
        Ok(
            Translator::init(&YUNOHOST_GLOBAL_I18N, YUNOHOST_LOCALES_DIR.into())?
                .write()
                .unwrap(),
        )
    }
}

/// Gets a Yunohost translation with a given context.
///
/// This method substitutes key names in the context HashMap
/// with the corresponding values in the formatted translation.
///
/// For example, for a translation key `user_not_found` containing the translation string:
/// `User was not found: {username}`, `yunohost_context("user_not_found", hashmap!{"username": "toto"})`
/// will produce the string `User was not found: toto`.
pub fn yunohost_context(key: &str, context: HashMap<String, String>) -> Result<String, Error> {
    yunohost_load()?.translate_with_context(key, context)
}

/// Gets a Yunohost translation without context (variable substitution)
pub fn yunohost_no_context(key: &str) -> Result<String, Error> {
    yunohost_load()?.translate_no_context(key)
}

/// Checks whether a translation key exists in Yunohost strings
pub fn yunohost_exists(key: &str) -> Result<bool, Error> {
    Ok(yunohost_load()?.key_exists(key))
}

/// Get the currently enabled locale
pub fn locale_get() -> Result<String, Error> {
    Ok(yunohost_load()?.get_locale())
}

/// Set the enabled locale to a new value
// TODO make this operation fallible so we never fail to find the locale when translating
pub fn locale_set(locale: &str) -> Result<(), Error> {
    yunohost_load_mut()?.set_locale(locale);
    moulinette_load_mut()?.set_locale(locale);
    Ok(())
}
