use clap::{Parser, Subcommand};
use log::LevelFilter;

use std::process::exit;

use yunohost::{error::*, helpers::file::*, helpers::output::*, helpers::regenconf::*};

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Enable json output
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SubCommand {
    #[command(name = "regen-conf")]
    RegenConf(RegenConfCommand),
}

#[derive(Clone, Debug, Parser)]
pub struct RegenConfCommand {
    #[arg(long = "list-pending")]
    list_pending: bool,

    #[arg(long)]
    json: bool,

    #[arg()]
    names: Vec<String>,
}

impl RegenConfCommand {
    pub fn run(&self, json: bool) -> Result<(), Error> {
        if path_exists("/etc/yunohost/settings.json") && !path_exists("/etc/yunohost/settings.yml")
        {
            eprintln!("This regenconf version can only run after 0025_global_settings_to_configpanel migration.");
            exit(1);
        }

        if self.list_pending {
            let pending = _get_pending_conf(&self.names)?;
            println!("{}", json_or_yaml_output(&pending, json)?);
        } else {
            println!("No command");
            exit(1);
        }

        Ok(())
    }
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
        SubCommand::RegenConf(regenconf_cmd) => {
            regenconf_cmd.run(cli.json)?;
        }
    }

    Ok(())
}
