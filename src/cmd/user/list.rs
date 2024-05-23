use clap::Parser;
use serde::Serialize;

use std::collections::BTreeMap;

use crate::{
    error::*,
    helpers::output,
    helpers::user::{UserAttr, YunohostUser},
};

#[derive(Clone, Debug, Parser)]
pub struct UserListCommand {
    #[arg(long)]
    json: bool,

    #[arg(long)]
    fields: Vec<UserAttr>,
}

impl UserListCommand {
    pub fn run(&self) -> Result<(), Error> {
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
