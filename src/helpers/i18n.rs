use toml::Table;

use crate::moulinette::i18n;

// This table apparenlty is inside ask key:
//
// fr: "Quelque chose ?"
// en: "Something ?"
//
// We try to find a value for the current locale, or the default locale
// worst case we take the first entry
pub fn _value_for_locale(values: &Table) -> String {
    let current_locale = i18n::get_locale();

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

    // TODO: fallback to first value ?
    // This should really not happen
    unimplemented!();
}
