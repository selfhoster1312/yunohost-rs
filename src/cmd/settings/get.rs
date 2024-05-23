use clap::Parser;

use crate::{
    error::*,
    helpers::{configpanel::*, legacy::*, output, settings::*},
};

#[derive(Clone, Debug, Parser)]
pub struct SettingsGetCommand {
    #[arg(short, long, name = "export")]
    export: bool,

    #[arg(short, long, name = "full")]
    full: bool,

    #[arg(long)]
    json: bool,

    #[arg()]
    setting: String,
}

impl SettingsGetCommand {
    pub fn run(&self) -> Result<(), Error> {
        if self.json {
            output::enable_json();
        }

        if self.full && self.export {
            return Err(Error::SettingsNoExportAndFull);
        }

        let mode = if self.full {
            GetMode::Full
        } else if self.export {
            GetMode::Export
        } else {
            GetMode::Classic
        };

        let mut settings = SettingsConfigPanel::new();

        let key = translate_legacy_settings_to_configpanel_settings(&self.setting);

        output::exit_result_output(settings.get(key, mode));

        Ok(())
    }
}
