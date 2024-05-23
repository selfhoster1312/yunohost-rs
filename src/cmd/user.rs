use clap::{Parser, Subcommand};
use serde::Serialize;

use std::collections::BTreeMap;

use crate::{
    error::*,
    helpers::mail::*,
    helpers::output,
    helpers::user::{UserAttr, UserQuery, YunohostUser},
};

#[derive(Clone, Debug, Parser)]
pub struct UserCommand {
    #[command(subcommand)]
    cmd: UserSubCommand,
}

impl UserCommand {
    pub fn run(&self) -> Result<(), Error> {
        match &self.cmd {
            UserSubCommand::UserInfo(cmd) => cmd.run(),
            UserSubCommand::UserList(cmd) => cmd.run(),
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum UserSubCommand {
    #[command(name = "info")]
    UserInfo(UserInfoCommand),
    #[command(name = "list")]
    UserList(UserListCommand),
}

#[derive(Clone, Debug, Parser)]
pub struct UserInfoCommand {
    #[arg(long)]
    json: bool,

    #[arg()]
    query: UserQuery,
}

impl UserInfoCommand {
    fn run(&self) -> Result<(), Error> {
        if self.json {
            output::enable_json();
        }

        // let fields = vec!(
        //     UserAttr::Fullname,
        //     UserAttr::Mail,
        //     UserAttr::Username,
        //     UserAttr::MailAlias,
        //     UserAttr::MailboxQuota,
        //     UserAttr::Shell,
        // );

        // Get the user from the LDAP DB
        let user = YunohostUser::get(self.query.clone())?;

        // Transform for extra fields of interest
        let user = DefaultSingle::try_from(user)?;

        // Format the output
        let output = output::format(&user)?;
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
        if self.json {
            output::enable_json();
        }

        // Get the userlist from the LDAP DB
        let users = YunohostUser::list(None)?;

        // Extract the fields that interest us
        let users = DefaultList::from(users);

        // Format the output
        let output = output::format(&users)?;
        println!("{}", output);

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DefaultListInfo {
    pub username: String,
    pub fullname: String,
    pub mail: String,
    #[serde(rename = "mailbox-quota")]
    pub mailbox_quota: String,
}

impl From<YunohostUser> for DefaultListInfo {
    fn from(user: YunohostUser) -> Self {
        Self {
            username: user.username,
            fullname: user.fullname,
            mail: user.mail,
            mailbox_quota: user.mailbox_quota,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DefaultList {
    users: BTreeMap<String, DefaultListInfo>,
}

impl From<Vec<YunohostUser>> for DefaultList {
    fn from(users: Vec<YunohostUser>) -> Self {
        let mut default_list: BTreeMap<String, DefaultListInfo> = BTreeMap::new();
        for user in users {
            default_list.insert(user.username.clone(), user.into());
        }

        Self {
            users: default_list,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DefaultSingle {
    pub username: String,
    pub fullname: String,
    pub mail: String,
    #[serde(rename = "loginShell")]
    pub login_shell: String,
    #[serde(rename = "mail-aliases")]
    pub mail_aliases: Vec<String>,
    #[serde(rename = "mail-forward")]
    pub mail_forward: Vec<String>,
    #[serde(rename = "mailbox-quota")]
    pub mailbox_quota: MailStorageUse,
}

impl TryFrom<YunohostUser> for DefaultSingle {
    type Error = Error;

    fn try_from(user: YunohostUser) -> Result<Self, Error> {
        let mailbox_quota =
            MailStorageUse::from_name_and_quota(&user.username, &user.mailbox_quota)?;
        Ok(Self {
            username: user.username,
            fullname: user.fullname,
            mail: user.mail,
            login_shell: user.login_shell,
            mail_aliases: user.mail_aliases,
            mail_forward: user.mail_forward,
            mailbox_quota,
        })
    }
}
