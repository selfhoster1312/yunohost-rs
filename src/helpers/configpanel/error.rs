use snafu::prelude::*;

use super::filter_key::FilterKey;
use crate::helpers::file::{error::FileError, StrPath};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ConfigPanelError {
    // version.rs (ConfigPanelVersion::from_str, ConfigPanelVersion::from_f64)
    // Python: ValueError(f"Config panels version '{value}' are no longer supported.")
    #[snafu(display("ConfigPanel version {version} is no longer supported"))]
    ConfigPanelVersion { version: String },

    // mod.rs (ConfigPanel::new)
    // Python: YunohostValidationError("config_no_panel") <-- TODO
    #[snafu(display("ConfigPanel {entity}: failed to read config file {path}"))]
    ConfigPanelConfigRead {
        entity: String,
        path: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // mod.rs (ConfigPanel::saved_settings)
    // Python: ???
    #[snafu(display("ConfigPanel {entity}: failed to read save file {path}"))]
    ConfigPanelSaveRead {
        entity: String,
        path: StrPath,
        #[snafu(source(from(FileError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // filter_key.rs (FilterKey::from_str)
    // Python: YunohostError(f"The filter key {key} has too many sub-levels, the max is 3.")
    #[snafu(display("FilterKey cannot have so many depth levels (3 max): {filter_key}"))]
    FilterKeyTooDeep { filter_key: String },

    // filter_key.rs (FilterKey::from_str)
    // Python: not an error
    #[snafu(display("FilterKey cannot be empty"))]
    FilterKeyNone,

    // mod.rs (ConfigPanel::get_single, ConfigPanel::get_multi)
    // Python: YunohostValidationError("config_unknown_filter_key", filter_key=self.filter_key) <-- TODO
    #[snafu(display("ConfigPanel {entity}: FilterKey {filter_key} not found"))]
    FilterKeyNotFound {
        entity: String,
        filter_key: FilterKey,
    },

    // mod.rs (ConfigPanel::to_compact/ConfigPanel::get_single)
    // Python: ????
    #[snafu(display("Invalid option type for option {option_id}: {option_type}"))]
    OptionTypeWrong {
        option_id: String,
        option_type: String,
        source: strum::ParseError,
    },
}

impl std::cmp::PartialEq for ConfigPanelError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::ConfigPanelVersion { version: v1 },
                Self::ConfigPanelVersion { version: v2 },
            ) => v1 == v2,
            (
                Self::ConfigPanelConfigRead {
                    entity: entity1,
                    path: path1,
                    ..
                },
                Self::ConfigPanelConfigRead {
                    entity: entity2,
                    path: path2,
                    ..
                },
            ) => entity1 == entity2 && path1 == path2,
            (
                Self::ConfigPanelSaveRead {
                    entity: entity1,
                    path: path1,
                    ..
                },
                Self::ConfigPanelSaveRead {
                    entity: entity2,
                    path: path2,
                    ..
                },
            ) => entity1 == entity2 && path1 == path2,
            (
                Self::FilterKeyTooDeep { filter_key: f1 },
                Self::FilterKeyTooDeep { filter_key: f2 },
            ) => f1 == f2,
            (Self::FilterKeyNone, Self::FilterKeyNone) => true,
            (
                Self::FilterKeyNotFound {
                    entity: entity1,
                    filter_key: f1,
                },
                Self::FilterKeyNotFound {
                    entity: entity2,
                    filter_key: f2,
                },
            ) => entity1 == entity2 && f1 == f2,
            (
                Self::OptionTypeWrong {
                    option_id: id1,
                    option_type: t1,
                    ..
                },
                Self::OptionTypeWrong {
                    option_id: id2,
                    option_type: t2,
                    ..
                },
            ) => id1 == id2 && t1 == t2,
            _ => false,
        }
    }
}
