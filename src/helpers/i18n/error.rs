use snafu::prelude::*;

use std::collections::HashMap;

use crate::helpers::file::{error::FileError, StrPath};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum I18NError {
    #[snafu(display("Failed to read the locales from {}", path))]
    LocalesReadFailed {
        path: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("Missing translation key for locale {locale}: {key}"))]
    LocalesMissingKey { key: String, locale: String },

    #[snafu(display(
        "Failed to format locale {locale} translation key {key} with the args:\n{:?}",
        args
    ))]
    LocalesFormatting {
        locale: String,
        key: String,
        args: Option<HashMap<String, String>>,
        source: strfmt::FmtError,
    },
}
