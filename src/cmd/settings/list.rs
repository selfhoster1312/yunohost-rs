use clap::Parser;
use snafu::prelude::*;

use crate::{
    error::*,
    helpers::{configpanel::GetMode, output, settings::SettingsConfigPanel},
};

#[derive(Clone, Debug, Parser)]
pub struct SettingsListCommand {
    #[arg(short, long)]
    full: bool,

    #[arg(short, long)]
    export: bool,

    #[arg(long)]
    json: bool,
}

impl SettingsListCommand {
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
        let res = settings.list(mode).context(ConfigPanelSnafu);
        output::exit_result_output(res);

        Ok(())
    }
}
