use camino::Utf8Path;
use snafu::prelude::*;

use std::collections::HashMap;
use std::sync::OnceLock;

use super::{
    get_system_locale, MaybeTranslation, RwLock, Translation, TranslatorInterface,
    DEFAULT_LOCALE_FALLBACK,
};
use crate::{error::*, helpers::file::*};

#[derive(Debug)]
pub(crate) struct Translator {
    locale: String,
    translations: HashMap<String, RwLock<MaybeTranslation>>,
}

impl Translator {
    pub(crate) fn init(
        state: &'static OnceLock<RwLock<Box<dyn TranslatorInterface>>>,
        locales_dir: &Utf8Path,
    ) -> Result<&'static RwLock<Box<dyn TranslatorInterface>>, Error> {
        let locale = get_system_locale();

        if let Some(translator) = state.get() {
            Ok(translator)
        } else {
            let translator = Translator::new(locales_dir, &locale)?;
            Ok(state.get_or_init(|| RwLock::new(Box::new(translator))))
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
}

impl TranslatorInterface for Translator {
    fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }

    fn get_locale(&self) -> String {
        self.locale.to_string()
    }

    fn key_exists(&self, key: &str) -> bool {
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

    fn translate_no_context(&self, key: &str) -> Result<String, Error> {
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

    fn translate_with_context(
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
