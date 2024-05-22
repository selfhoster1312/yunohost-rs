use ldap3::{Scope, SearchEntry};
use snafu::prelude::*;
use tokio::runtime::Builder as RuntimeBuilder;

use std::str::FromStr;

use crate::{
    error::*,
    helpers::{file::read, ldap::*, process::*},
};

pub struct YunohostGroup;

impl YunohostGroup {
    /// Checks whether a POSIX group exists.
    ///
    /// Errors when:
    ///   - reading /etc/group failed
    pub fn exists(name: &str) -> Result<bool, Error> {
        let expected = format!("{name}:");
        Ok(read("/etc/group")
            .context(YunohostGroupExistsReadSnafu {
                name: name.to_string(),
            })?
            .lines()
            .any(|line| line.starts_with(&expected)))
    }

    /// Creates a POSIX group on the system.
    ///
    /// Errors when:
    ///   - reading /etc/group failed
    ///   - group `name` already exists
    ///   - groupadd command fails
    pub fn add(name: &str) -> Result<(), Error> {
        if Self::exists(name)? {
            return Err(Error::YunohostGroupExists {
                name: name.to_string(),
            });
        }

        if !cmd("groupadd", vec![name]).unwrap().status.success() {
            return Err(Error::YunohostGroupCreate {
                name: name.to_string(),
            });
        }

        Ok(())
    }

    /// Make sure a POSIX group exists on the system.
    ///
    /// Errors when:
    ///   - reading /etc/group failed
    ///   - group `name` did not exist and groupadd command failed
    ///
    /// Does not error when the group does not exist.
    pub fn ensure_exists(name: &str) -> Result<(), Error> {
        if Self::exists(name)? {
            return Ok(());
        }

        if !cmd("groupadd", vec![name]).unwrap().status.success() {
            return Err(Error::YunohostGroupCreate {
                name: name.to_string(),
            });
        }

        Ok(())
    }
}

/// A specific user to query information about.
///
/// Can work with queries by username or email.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserQuery {
    Username(String),
    Mail(String),
}

impl<T: AsRef<str>> From<T> for UserQuery {
    fn from(s: T) -> UserQuery {
        let s = s.as_ref();
        if s.contains('@') {
            UserQuery::Mail(s.to_string())
        } else {
            UserQuery::Username(s.to_string())
        }
    }
}

impl UserQuery {
    pub fn to_ldap_filter(&self) -> String {
        match self {
            UserQuery::Username(s) => {
                format!("uid={}", s)
            }
            UserQuery::Mail(s) => {
                format!("mail={}", s)
            }
        }
    }
}

/// A permission on the Yunohost system, with associated users.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YunohostPermission {
    pub name: String,
    pub corresponding_users: Vec<String>,
}

impl YunohostPermission {
    pub fn name_from_dn(dn: &str) -> String {
        dn.trim_start_matches("cn=")
            .trim_end_matches(",ou=permission,dc=yunohost,dc=org")
            .to_string()
    }

    pub fn get(name: &str) -> Result<Self, Error> {
        let permissions = Self::list()?;

        for perm in permissions {
            if perm.name == name {
                return Ok(perm);
            }
        }

        Err(Error::LdapPermissionNotFound {
            name: name.to_string(),
        })
    }

    pub fn list() -> Result<Vec<Self>, Error> {
        let rt = RuntimeBuilder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let permissions_list = rt.block_on(async {
            let ldap = YunohostLDAP::new(1000).await?;

            let attrs: Vec<&'static str> = vec![
                "cn",
                "groupPermission",
                "inheritPermission",
                "URL",
                "additionalUrls",
                "authHeader",
                "label",
                "showTile",
                "isProtected",
            ];

            let permissions = ldap
                .list(
                    "ou=permission,dc=yunohost,dc=org",
                    Scope::OneLevel,
                    "(objectclass=permissionYnh)",
                    attrs,
                )
                .await?;

            Ok(permissions)
        })?;

        let mut new_list: Vec<YunohostPermission> = vec![];
        for entry in permissions_list {
            new_list.push(Self::try_from(entry)?);
        }

        Ok(new_list)
    }
}

impl TryFrom<SearchEntry> for YunohostPermission {
    type Error = Error;

    fn try_from(perm: SearchEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            name: Self::name_from_dn(&perm.dn),
            // inheritPermission was requested so this should be safe unwrap
            corresponding_users: perm
                .attrs
                .get("inheritPermission")
                .map(|found_list| {
                    found_list
                        .into_iter()
                        .map(|dn| YunohostUser::name_from_dn(&dn))
                        .collect()
                })
                .unwrap_or(vec![]),
        })
    }
}

/// A user on the Yunohost system.
///
/// More specifically, an entry with `username` *uid* in the `ou=users` in the
/// Yunohost LDAP database.
///
/// The user information is populated only with requested [`UserAttr`] attributes, so make sure they are requested
/// when loading the users from LDAP. Only the `username` user field (`uid` attribute) is mandatory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YunohostUser {
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
    pub mailbox_quota: String,
}

impl YunohostUser {
    pub fn get<T: Into<UserQuery>>(query: T) -> Result<Self, Error> {
        // Support name and mail
        let query: UserQuery = query.into();

        let rt = RuntimeBuilder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let user = rt.block_on(async {
            let ldap = YunohostLDAP::new(1000).await?;

            let attrs: Vec<&'static str> = vec![
                UserAttr::Fullname,
                UserAttr::Mail,
                UserAttr::Username,
                UserAttr::MailForward,
                UserAttr::MailboxQuota,
                UserAttr::Shell,
            ]
            .into_iter()
            .map(|attr| attr.to_ldap_attr())
            .collect();

            let user = ldap
                .search(
                    &format!("{},ou=users,dc=yunohost,dc=org", query.to_ldap_filter()),
                    Scope::Base,
                    "(objectclass=*)",
                    attrs,
                    Error::LdapNoSuchUser {
                        query: query.clone(),
                    },
                )
                .await?;

            Ok(user)
        })?;

        Ok(YunohostUser::try_from(user)?)
    }

    pub fn list(attrs: Option<Vec<UserAttr>>) -> Result<Vec<Self>, Error> {
        // Default attributes, unless some attrs were requested
        let attrs: Vec<String> = attrs
            .unwrap_or(vec![
                UserAttr::Fullname,
                UserAttr::Mail,
                UserAttr::Username,
                UserAttr::MailForward,
                UserAttr::MailboxQuota,
                UserAttr::Shell,
            ])
            .into_iter()
            .map(|attr| attr.to_ldap_attr().to_string())
            .collect();

        let rt = RuntimeBuilder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let user_list = rt.block_on(async {
            let ldap = YunohostLDAP::new(1000).await?;

            let user_list = ldap
                .list(
                    "ou=users,dc=yunohost,dc=org",
                    Scope::OneLevel,
                    "(&(objectclass=person)(!(uid=root))(!(uid=nobody)))",
                    attrs,
                )
                .await?;

            Ok(user_list)
        })?;

        let mut new_list: Vec<YunohostUser> = vec![];
        for entry in user_list {
            new_list.push(entry.try_into()?);
        }

        Ok(new_list)
    }

    /// Shorthand method for querying the list of usernames only
    pub fn usernames() -> Result<Vec<String>, Error> {
        Ok(Self::list(None)?.into_iter().map(|x| x.username).collect())
    }

    pub fn name_from_dn(dn: &str) -> String {
        dn.trim_start_matches("uid=")
            .trim_end_matches(",ou=users,dc=yunohost,dc=org")
            .to_string()
    }
}

impl TryFrom<SearchEntry> for YunohostUser {
    type Error = Error;

    fn try_from(user: SearchEntry) -> Result<Self, Self::Error> {
        let mut mail_aliases: Vec<String> = vec![];
        for addr in user
            .attrs
            .get(UserAttr::MailAlias.to_ldap_attr())
            .unwrap()
            .into_iter()
            .skip(1)
        {
            mail_aliases.push(addr.to_string());
        }

        let mut mail_forward: Vec<String> = vec![];
        for addr in user
            .attrs
            .get(UserAttr::MailForward.to_ldap_attr())
            .unwrap()
            .into_iter()
            .skip(1)
        {
            mail_forward.push(addr.to_string());
        }

        Ok(Self {
            username: Self::name_from_dn(&user.dn),
            fullname: user
                .attrs
                .get(UserAttr::Fullname.to_ldap_attr())
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .to_string(),
            mail: user
                .attrs
                .get(UserAttr::Mail.to_ldap_attr())
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .to_string(),
            login_shell: user
                .attrs
                .get(UserAttr::Shell.to_ldap_attr())
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .to_string(),
            mail_aliases,
            mail_forward,
            mailbox_quota: user
                .attrs
                .get(UserAttr::MailboxQuota.to_ldap_attr())
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .to_string(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserAttr {
    Username,
    Fullname,
    Firstname,
    Lastname,
    Mail,
    MailAlias,
    MailForward,
    MailboxQuota,
    Groups,
    Shell,
    HomePath,
}

impl FromStr for UserAttr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "username" => Ok(Self::Username),
            "password" => Err(Error::LdapUserAttrNotPassword),
            "fullname" => Ok(Self::Fullname),
            "firstname" => Ok(Self::Firstname),
            "lastname" => Ok(Self::Lastname),
            "mail" => Ok(Self::Mail),
            "mail-alias" => Ok(Self::MailAlias),
            "mail-forward" => Ok(Self::MailForward),
            "mailbox-quota" => Ok(Self::MailboxQuota),
            "groups" => Ok(Self::Groups),
            "shell" => Ok(Self::Shell),
            "home-path" => Ok(Self::HomePath),
            _ => Err(Error::LdapUserAttrUnknown {
                field: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for UserAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let field = match self {
            Self::Username => "username",
            Self::Fullname => "fullname",
            Self::Firstname => "firstname",
            Self::Lastname => "lastname",
            Self::Mail => "mail",
            Self::MailAlias => "mail-alias",
            Self::MailForward => "mail-forward",
            Self::MailboxQuota => "mailbox-quota",
            Self::Groups => "groups",
            Self::Shell => "shell",
            Self::HomePath => "home-path",
        };

        write!(f, "{}", field)
    }
}

impl UserAttr {
    pub fn to_ldap_attr(&self) -> &'static str {
        match self {
            Self::Username => "uid",
            Self::Fullname => "cn",
            Self::Firstname => "givenName",
            Self::Lastname => "sn",
            Self::Mail => "mail",
            Self::MailAlias => "mail",
            Self::MailForward => "maildrop",
            Self::MailboxQuota => "mailuserquota",
            Self::Groups => "memberOf",
            Self::Shell => "loginShell",
            Self::HomePath => "homeDirectory",
        }
    }
}
