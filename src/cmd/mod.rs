use clap::Subcommand;
// use clap::Parser;

use crate::error::*;

pub mod hook;
use hook::HookCommand;
pub mod tools;
use tools::ToolsCommand;
pub mod user;
use user::UserCommand;
pub mod settings;
use settings::SettingsCommand;

#[derive(Clone, Debug, Subcommand)]
pub enum YunohostCommand {
    // #[command(name="hook")]
    // Hook {
    //     #[command(subcommand)]
    //     cmd: HookCommand,
    // },
    // #[command(name="tools")]
    // Tools {
    //     #[command(subcommand)]
    //     cmd: ToolsCommand,
    // },
    // #[command(name="user")]
    // User {
    //     #[command(subcommand)]
    //     cmd: UserCommand,
    // },
    // #[command(name="settings")]
    // Settings {
    //     #[command(subcommand)]
    //     cmd: SettingsCommand
    // },
    #[command(name = "hook")]
    Hook(HookCommand),
    #[command(name = "tools")]
    Tools(ToolsCommand),
    #[command(name = "user")]
    User(UserCommand),
    #[command(name = "settings")]
    Settings(SettingsCommand),
}

impl YunohostCommand {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            // YunohostCommand::Hook{cmd} => cmd.run(),
            // YunohostCommand::User{cmd} => cmd.run(),
            // YunohostCommand::Settings{cmd} => cmd.run(),
            // YunohostCommand::Tools{cmd} => cmd.run(),
            YunohostCommand::Hook(cmd) => cmd.run(),
            YunohostCommand::User(cmd) => cmd.run(),
            YunohostCommand::Settings(cmd) => cmd.run(),
            YunohostCommand::Tools(cmd) => cmd.run(),
        }
    }
}
