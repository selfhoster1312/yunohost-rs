//! # Moulinette18n
//!
//! This module contains helpers for internationalization. Yunohost translations are contained in one JSON file per locale,
//! in the /usr/share/{yunohost,moulinette}/locales/ directories.
//!
//! There are separate methods for accessing `yunohost` and `moulinette` translations, and they are prefixed accordingly.
//!
//! Contrary to the Python version, you do not need to explicitly initialize the translation system before you start asking for
//! translations. The translations will be loaded (just once) the first time you request them.

// use snafu::prelude::*;

use std::boxed::Box;
use std::collections::HashMap;
use std::env;
// ASYNC TODO: Replace std::sync::RwLock with tokio::sync::RwLock if we ever go async
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(crate) mod error;
use error::*;
mod mocked_translator;
use mocked_translator::MockedTranslator;
mod translator;
use translator::Translator;
mod value;
pub use value::_value_for_locale;

pub(crate) static DEFAULT_LOCALE_FALLBACK: &'static str = "en";

// Wrap in OnceLock to make sure it's only initialized once. Wrap in RwLock to allow inner mutability (Translator::set_locale)
pub(crate) static YUNOHOST_GLOBAL_I18N: OnceLock<RwLock<Box<dyn TranslatorInterface>>> =
    OnceLock::new();
pub(crate) static YUNOHOST_LOCALES_DIR: &'static str = "/usr/share/yunohost/locales";

pub(crate) static MOULINETTE_GLOBAL_I18N: OnceLock<RwLock<Box<dyn TranslatorInterface>>> =
    OnceLock::new();
pub(crate) static MOULINETTE_LOCALES_DIR: &'static str = "/usr/share/moulinette/locales";

type Translation = HashMap<String, String>;

#[derive(Debug)]
enum MaybeTranslation {
    NotLoaded(camino::Utf8PathBuf),
    Loaded(Translation),
}

/// A type that supports translation methods.
///
/// Used for actual translations with [`Translator`] and for tests with [`MockedTranslator`].
pub(crate) trait TranslatorInterface: std::fmt::Debug + Send + Sync {
    fn set_locale(&mut self, locale: &str);
    fn get_locale(&self) -> String;
    fn key_exists(&self, key: &str) -> bool;
    fn translate_no_context(&self, key: &str) -> Result<String, I18NError>;
    fn translate_with_context(
        &self,
        key: &str,
        context: HashMap<String, String>,
    ) -> Result<String, I18NError>;
}

pub(crate) fn get_system_locale() -> String {
    let locale = env::var("LC_ALL").or_else(|_| env::var("LANG")).unwrap();

    locale.chars().take(2).collect()
}

pub(crate) fn moulinette_load(
) -> Result<RwLockReadGuard<'static, Box<dyn TranslatorInterface>>, I18NError> {
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

pub(crate) fn moulinette_load_mut(
) -> Result<RwLockWriteGuard<'static, Box<dyn TranslatorInterface>>, I18NError> {
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
pub fn moulinette_context(
    key: &str,
    context: HashMap<String, String>,
) -> Result<String, I18NError> {
    let translator = moulinette_load()?;
    translator.translate_with_context(key, context)
}

/// Gets a Moulinette translation without context (variable substitution)
pub fn moulinette_no_context(key: &str) -> Result<String, I18NError> {
    let translator = moulinette_load()?;
    translator.translate_no_context(key)
}

/// Checks whether a translation key exists in Moulinette strings
pub fn moulinette_exists(key: &str) -> Result<bool, I18NError> {
    let translator = moulinette_load()?;
    Ok(translator.key_exists(key))
}

pub(crate) fn yunohost_load(
) -> Result<RwLockReadGuard<'static, Box<dyn TranslatorInterface>>, I18NError> {
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

pub(crate) fn yunohost_load_mut(
) -> Result<RwLockWriteGuard<'static, Box<dyn TranslatorInterface>>, I18NError> {
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
pub fn yunohost_context(key: &str, context: HashMap<String, String>) -> Result<String, I18NError> {
    yunohost_load()?.translate_with_context(key, context)
}

/// Gets a Yunohost translation without context (variable substitution)
pub fn yunohost_no_context(key: &str) -> Result<String, I18NError> {
    yunohost_load()?.translate_no_context(key)
}

/// Checks whether a translation key exists in Yunohost strings
pub fn yunohost_exists(key: &str) -> Result<bool, I18NError> {
    Ok(yunohost_load()?.key_exists(key))
}

/// Get the currently enabled locale
pub fn locale_get() -> Result<String, I18NError> {
    Ok(yunohost_load()?.get_locale())
}

/// Set the enabled locale to a new value
// TODO make this operation fallible so we never fail to find the locale when translating
pub fn locale_set(locale: &str) -> Result<(), I18NError> {
    yunohost_load_mut()?.set_locale(locale);
    moulinette_load_mut()?.set_locale(locale);
    Ok(())
}

/// Initialize the I18N test system for test runners.
// TODO: Is this state shared across tests??? Why would it be???
// If it is none, why is sometimes YUNOHOST_GLOBAL_I18N already set?
#[allow(dead_code)]
pub(crate) fn test_init() {
    if YUNOHOST_GLOBAL_I18N.get().is_none() {
        YUNOHOST_GLOBAL_I18N
            .set(RwLock::new(Box::new(MockedTranslator::new())))
            .unwrap();
    }

    if MOULINETTE_GLOBAL_I18N.get().is_none() {
        MOULINETTE_GLOBAL_I18N
            .set(RwLock::new(Box::new(MockedTranslator::new())))
            .unwrap();
    }
}
