use clap::{Parser, Subcommand};
use log::LevelFilter;

use yunohost::{
    error::*,
    helpers::output::*,
    helpers::users::{UserAttr, UserQuery, YunohostUserInfo, YunohostUsers},
    moulinette::i18n,
};

#[derive(Clone, Debug, Parser)]
pub struct UserInfoCommand {
    #[arg(long)]
    json: bool,

    #[arg()]
    query: UserQuery,
}

impl UserInfoCommand {
    fn run(&self) -> Result<(), Error> {
        // let fields = vec!(
        //     UserAttr::Fullname,
        //     UserAttr::Mail,
        //     UserAttr::Username,
        //     UserAttr::MailAlias,
        //     UserAttr::MailboxQuota,
        //     UserAttr::Shell,
        // );

        let user = YunohostUserInfo::info_for(self.query.clone())?;
        let output = json_or_yaml_output(&user, self.json)?;
        println!("{}", output);

        Ok(())
    }
}

#[derive(Clone, Debug, Parser)]
pub struct UserListCommand {
    #[arg(long)]
    json: bool,

    #[arg(long)]
    fields: Vec<UserAttr>,
}

impl UserListCommand {
    fn run(&self) -> Result<(), Error> {
        // TODO: custom fields
        let users = YunohostUsers::default_list()?;
        let output = json_or_yaml_output(&users, self.json)?;
        println!("{}", output);

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
    #[command(name = "info")]
    UserInfo(UserInfoCommand),
    #[command(name = "list")]
    UserList(UserListCommand),
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

    i18n::init()?;

    match cli.command {
        SubCommand::UserInfo(userget_cmd) => {
            userget_cmd.run()?;
        }
        SubCommand::UserList(userlist_cmd) => {
            userlist_cmd.run()?;
        }
    }

    Ok(())
}
