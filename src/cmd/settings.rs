use clap::{Parser, Subcommand};

use crate::{
    error::*,
    helpers::{configpanel::*, legacy::*, output, settings::*},
};

#[derive(Clone, Debug, Parser)]
pub struct SettingsCommand {
    #[command(subcommand)]
    cmd: SettingsSubCommand,
}

impl SettingsCommand {
    pub fn run(&self) -> Result<(), Error> {
        match &self.cmd {
            SettingsSubCommand::SettingsGet(cmd) => cmd.run(),
            SettingsSubCommand::SettingsList(cmd) => cmd.run(),
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum SettingsSubCommand {
    #[command(name = "get")]
    SettingsGet(SettingsGetCommand),
    #[command(name = "list")]
    SettingsList(SettingsListCommand),
}

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
    fn run(&self) -> Result<(), Error> {
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
        let val = settings.get(&key, mode);

        // println!("{}", json_or_yaml_output(&val, self.json)?);
        output::fallible(val);

        Ok(())
    }
}

#[derive(Clone, Debug, Parser)]
pub struct SettingsListCommand {
    #[arg(short, long, name = "full")]
    _full: bool,

    #[arg(long)]
    json: bool,

    #[arg()]
    setting: String,
}

impl SettingsListCommand {
    fn run(&self) -> Result<(), Error> {
        if self.json {
            output::enable_json();
        }

        Ok(())
    }
}
