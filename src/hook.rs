use clap::{Parser, Subcommand};
use log::LevelFilter;

use yunohost::{
    error::*,
    helpers::{hook::*, output::*},
};

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
    List {
        /// Enable json output
        #[arg(long)]
        json: bool,

        #[arg()]
        action: String,
    },
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
        SubCommand::List { action, json } => {
            let list = HookList::for_action(&action);
            println!("{}", json_or_yaml_output(&list.names(), json)?);
        }
    }

    Ok(())
}
