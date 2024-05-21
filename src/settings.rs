use clap::{Parser, Subcommand};
use log::LevelFilter;

use yunohost::{
    error::*,
    helpers::{configpanel::*, legacy::*, output::*, settings::*},
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
    fn run(&self) -> Result<(), Error> {
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
        fallible_output(val, self.json);

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
        Ok(())
    }
}

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SubCommand {
    #[command(name = "get")]
    SettingsGet(SettingsGetCommand),
    #[command(name = "list")]
    SettingsList(SettingsListCommand),
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    if cli.debug {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Debug)
            .init();
    } else {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Info)
            .init();
    }

    match cli.command {
        SubCommand::SettingsGet(cmd) => {
            cmd.run()?;
        }
        SubCommand::SettingsList(cmd) => {
            cmd.run()?;
        }
    }

    Ok(())
}
