use clap::Parser;
use log::LevelFilter;

use yunohost::{
    cmd::YunohostCommand,
    error::*,
    // helpers::output::*,
};

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct YunohostCli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: YunohostCommand,
}

fn main() -> Result<(), Error> {
    // Parse the typed CLI
    let cli = YunohostCli::parse();

    // Check whether to enable debug log
    if cli.debug {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Debug)
            .init();
    } else {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Info)
            .init();
    }

    cli.command.run()?;

    Ok(())
}
