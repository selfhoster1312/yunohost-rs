use clap::Parser;
use serde::Serialize;

use crate::{
    error::*,
    helpers::mail::*,
    helpers::output,
    helpers::user::{UserQuery, YunohostUser},
};

#[derive(Clone, Debug, Parser)]
pub struct UserInfoCommand {
    #[arg(long)]
    json: bool,

    #[arg()]
    query: UserQuery,
}

impl UserInfoCommand {
    pub fn run(&self) -> Result<(), Error> {
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
