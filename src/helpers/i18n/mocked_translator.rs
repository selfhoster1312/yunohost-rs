use std::collections::HashMap;

use super::{error::*, TranslatorInterface, DEFAULT_LOCALE_FALLBACK};

/// A fake translator where all translation keys exist and the locale is always the default locale.
#[derive(Debug)]
pub(crate) struct MockedTranslator;

impl MockedTranslator {
    pub fn new() -> MockedTranslator {
        MockedTranslator {}
    }
}

impl TranslatorInterface for MockedTranslator {
    fn set_locale(&mut self, _locale: &str) {}

    fn get_locale(&self) -> String {
        DEFAULT_LOCALE_FALLBACK.to_string()
    }

    fn key_exists(&self, _key: &str) -> bool {
        true
    }

    fn translate_no_context(&self, key: &str) -> Result<String, I18NError> {
        Ok(key.to_string())
    }

    fn translate_with_context(
        &self,
        key: &str,
        _context: HashMap<String, String>,
    ) -> Result<String, I18NError> {
        Ok(key.to_string())
    }
}
