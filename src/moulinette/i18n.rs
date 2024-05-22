// use camino::{Utf8PathBuf, Utf8Path};
use camino::Utf8Path;

use std::env;
use std::sync::{OnceLock, RwLock};

// pub const DEFAULT_MOULINETTE_TRANSLATIONS_STR = include_str!("../locales/en.json");
// pub const DEFAULT_YUNOHOST_TRANSLATIONS_STR = include_str!("../../locales/en.json");

// static DEFAULT_MOULINETTE_TRANSLATIONS: OnceLock<HashMap<String, String>> = OnceLock::new();
// static DEFAULT_YUNOHOST_TRANSLATIONS: OnceLock<HashMap<String, String>> = OnceLock::new();
use std::collections::HashMap;

#[allow(dead_code)]
pub static DEFAULT_LOCALE_FALLBACK: &'static str = "en";

// Wrap in OnceLock to make sure it's only initialized once. Wrap in RwLock to allow inner mutability.
pub static STATIC_I18N: OnceLock<RwLock<Moulinette18n>> = OnceLock::new();

pub struct Translator {
    // locale_dir: Utf8PathBuf,
    locale: String,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Translator {
    pub fn new(locale_dir: &Utf8Path, default_locale: &str) -> Translator {
        let mut translations: HashMap<String, HashMap<String, String>> = HashMap::new();
        for locale in locale_dir.read_dir_utf8().unwrap() {
            let locale = locale.unwrap();
            let file_name = locale.path().file_name().unwrap();

            if !file_name.ends_with(".json") {
                continue;
            }

            let locale_name = file_name.trim_end_matches(".json");
            let locale_values: HashMap<String, String> =
                serde_json::from_str(&std::fs::read_to_string(locale.path()).unwrap()).unwrap();

            translations.insert(locale_name.to_string(), locale_values);
        }

        Translator {
            // locale_dir: locale_dir.to_path_buf(),
            locale: default_locale.to_string(),
            translations,
        }
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

    pub fn translate(&self, key: &str, context: Option<HashMap<String, String>>) -> String {
        if let Some(val) = self.translations.get(&self.locale).unwrap().get(key) {
            if let Some(context) = context {
                return strfmt::strfmt(val, &context).unwrap();
            } else {
                return val.to_string();
            }
        }

        panic!();
    }
}

pub struct Moulinette18n {
    pub current_locale: String,
    pub yunohost_translator: Translator,
    pub moulinette_translator: Translator,
}

impl Moulinette18n {
    pub fn new(locale: &str) -> Moulinette18n {
        let yunohost_translator = Translator::new("/usr/share/yunohost/locales".into(), locale);
        let moulinette_translator = Translator::new("/usr/share/moulinette/locales".into(), locale);
        Moulinette18n {
            yunohost_translator,
            moulinette_translator,
            current_locale: locale.to_string(),
        }
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

pub fn init() -> &'static RwLock<Moulinette18n> {
    let locale = get_system_locale();

    STATIC_I18N.get_or_init(|| RwLock::new(Moulinette18n::new(&locale)))
}

pub fn g(key: &str, context: Option<HashMap<String, String>>) -> String {
    STATIC_I18N
        .get()
        .unwrap()
        .read()
        .unwrap()
        .moulinette_translator
        .translate(key, context)
}

pub fn n(key: &str, context: Option<HashMap<String, String>>) -> String {
    STATIC_I18N
        .get()
        .unwrap()
        .read()
        .unwrap()
        .yunohost_translator
        .translate(key, context)
}

pub fn get_locale() -> String {
    STATIC_I18N.get().unwrap().read().unwrap().get_locale()
}

pub fn set_locale(locale: &str) {
    let mut m18n = STATIC_I18N.get().unwrap().write().unwrap();
    m18n.set_locale(locale);
}
