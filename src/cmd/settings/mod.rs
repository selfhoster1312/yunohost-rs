use clap::{Parser, Subcommand};

use crate::error::Error;

pub mod get;
pub mod list;

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
    SettingsGet(get::SettingsGetCommand),
    #[command(name = "list")]
    SettingsList(list::SettingsListCommand),
}
