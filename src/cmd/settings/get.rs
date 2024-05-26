use clap::Parser;
use snafu::prelude::*;

use crate::{
    error::*,
    helpers::{configpanel::*, output, settings::*},
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
    setting: SettingsFilterKey,
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

        let mut settings = SettingsConfigPanel::new().context(ConfigPanelSnafu)?;
        let res = settings.get(&self.setting, mode).context(ConfigPanelSnafu);
        output::exit_result_output(res);

        Ok(())
    }
}
