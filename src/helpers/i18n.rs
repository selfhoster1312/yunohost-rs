use serde_json::{Map, Value};

use crate::moulinette::i18n;

/// Extract a single translation from a translation table.
///
/// First, we try to lookup the current locale. If that fails, we try the
/// locale. If that fails, we just take the first entry in the table.
///
/// Example:
/// ```rust
/// let translations: Map<String, Value> = serde_json::to_value(hashmap!(
///   "en".to_string() => "Install application?".to_string(),
///   "fr".to_string() => "Installer l'application?".to_string(),
/// )).unwrap();
/// println!("{}", _value_for_locale(&translations));
/// ```
pub fn _value_for_locale(values: &Map<String, Value>) -> String {
    // TODO: error condition
    let current_locale = i18n::locale_get().unwrap();

    if values.contains_key(&current_locale) {
        return values
            .get(&current_locale)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
    } else if values.contains_key(i18n::DEFAULT_LOCALE_FALLBACK) {
        return values
            .get(i18n::DEFAULT_LOCALE_FALLBACK)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
    }

    for (_key, value) in values {
        return value.as_str().unwrap().to_string();
    }

    unreachable!("empty translation table");
}

#[cfg(test)]
mod tests {
    use super::_value_for_locale;
    use crate::moulinette::i18n;
    use serde_json::{Map, Value};

    #[test]
    fn _value_for_locale_ok() {
        i18n::test_init();

        let translations: Map<String, Value> = serde_json::to_value(hashmap!(
          "en".to_string() => "Install application?".to_string(),
          "fr".to_string() => "Installer l'application?".to_string(),
        ))
        .unwrap()
        .as_object()
        .unwrap()
        .clone();

        // TODO: Here we are not mocking the locale and i18n so we are expecting either key...
        let expected: &'static [&'static str] =
            &["Install application?", "Installer l'application?"];
        let actual = _value_for_locale(&translations);
        if !expected.contains(&actual.as_str()) {
            panic!("Expected french or english translation, got: {actual}");
        }
    }

    #[test]
    fn _value_for_locale_default_fallback() {
        i18n::test_init();
        i18n::locale_set("fr").unwrap();

        let translations: Map<String, Value> = serde_json::to_value(hashmap!(
          "en".to_string() => "Install application?".to_string(),
        ))
        .unwrap()
        .as_object()
        .unwrap()
        .clone();

        assert_eq!(
            "Install application?",
            _value_for_locale(&translations).as_str()
        );
    }

    #[test]
    fn _value_for_locale_first_fallback() {
        i18n::test_init();
        i18n::locale_set("de").unwrap();

        let translations: Map<String, Value> = serde_json::to_value(hashmap!(
          "fr".to_string() => "Installer l'application ?".to_string(),
        ))
        .unwrap()
        .as_object()
        .unwrap()
        .clone();

        assert_eq!(
            "Installer l'application ?",
            _value_for_locale(&translations).as_str()
        );
    }
}
