use snafu::prelude::*;

use super::filter_key::FilterKey;
use crate::error::Error as YunohostError;
use crate::helpers::file::StrPath;

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
        #[snafu(source(from(YunohostError, Box::new)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // mod.rs (ConfigPanel::saved_settings)
    // Python: ???
    #[snafu(display("ConfigPanel {entity}: failed to read save file {path}"))]
    ConfigPanelSaveRead {
        entity: String,
        path: StrPath,
        #[snafu(source(from(YunohostError, Box::new)))]
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
}
