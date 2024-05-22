use regex::Regex;
use snafu::prelude::*;

use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::OnceLock;

pub static REGEX_MAILUSAGE: OnceLock<Regex> = OnceLock::new();

use crate::{
    error::*,
    helpers::{file::read, ldap::*, process::cmd, service::*},
    moulinette::i18n,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YunohostDefaultListInfo {
    pub username: String,
    pub fullname: String,
    pub mail: String,
    // TODO
    #[serde(rename = "mailbox-quota")]
    pub mailbox_quota: String,
}

impl YunohostDefaultListInfo {
    pub fn from_ldap_user(user: LdapUser) -> Self {
        Self {
            username: user.name,
            fullname: user.attrs.get("cn").unwrap().first().unwrap().to_string(),
            mail: user.attrs.get("mail").unwrap().first().unwrap().to_string(),
            mailbox_quota: user
                .attrs
                .get("mailuserquota")
                .unwrap()
                .first()
                .unwrap()
                .to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YunohostDefaultList {
    users: BTreeMap<String, YunohostDefaultListInfo>,
}

impl YunohostDefaultList {
    pub fn from_ldap_users(ldap_users: Vec<LdapUser>) -> Self {
        let mut users: BTreeMap<String, YunohostDefaultListInfo> = BTreeMap::new();

        for user in ldap_users {
            users.insert(
                user.name.clone(),
                YunohostDefaultListInfo::from_ldap_user(user),
            );
        }

        Self { users }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YunohostUsers {
    pub users: Vec<String>,
}

impl YunohostUsers {
    /// List existing POSIX usernames from the LDAP database.
    ///
    /// NOTE: This function creates a tokio async runtime. Do not use in async context.
    pub fn usernames() -> Result<Vec<String>, Error> {
        let mut users = vec![];
        for user in LdapUser::one_off_list_users(vec![UserAttr::Username.to_string()])? {
            users.push(user.name);
        }

        Ok(users)
    }

    pub fn default_list() -> Result<YunohostDefaultList, Error> {
        let attrs: Vec<&'static str> = vec![
            UserAttr::Username,
            UserAttr::Fullname,
            UserAttr::Mail,
            UserAttr::MailAlias,
            UserAttr::MailboxQuota,
            UserAttr::Shell,
        ]
        .into_iter()
        .map(|attr| attr.to_ldap_attr())
        .collect();

        let ldap_users = LdapUser::one_off_list_users(attrs)?;
        let users = YunohostDefaultList::from_ldap_users(ldap_users);

        Ok(users)
    }
}

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

// /// A user on the Yunohost system.
// ///
// /// More specifically, an entry with `username` *uid* in the `ou=users,dc=yunohost,dc=org` DN in the
// /// Yunohost LDAP database.
// ///
// /// The user information is populated only with requested [`UserAttr`] attributes, so make sure they are requested
// /// when loading the users from LDAP. Only the `username` user field (`uid` attribute) is mandatory.
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct YunohostUserInfo {
//     #[serde(skip)]
//     /// POSIX username, unique across all domains on the server
//     username: String,
//     #[serde(skip)]
//     /// List of user attributes/fields fetched from the database
//     fetched_attrs: Vec<UserAttr>,
//     #[serde(flatten)]
//     /// The attribute/field values returned from the database.
//     attrs: BTreeMap<UserAttr, Vec<String>>,
// }

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MailStorageUse {
    limit: String,
    #[serde(rename = "use")]
    mail_use: String,
}

impl MailStorageUse {
    pub fn from_name_and_quota(user: &str, quota: &str) -> Result<Self, Error> {
        let limit = if quota.starts_with("0") {
            i18n::n("unlimit", None)?
        } else {
            quota.to_string()
        };

        let mut mail_use = String::from("?");

        if !SystemCtl::is_active("dovecot") {
            warn!(
                "{}",
                i18n::n("mailbox_used_space_dovecot_down", None).context(
                    MailStorageLookupSnafu {
                        user: user.to_string(),
                    }
                )?
            );
        } else if !YunohostPermissionInfo::permission("mail.main")
            .context(MailStorageLookupSnafu {
                user: user.to_string(),
            })?
            .corresponding_users
            .contains(&user.to_string())
        {
            debug!(
                "{}",
                i18n::n(
                    "mailbox_disabled",
                    Some(hashmap!("user".to_string() => user.to_string()))
                )
                .context(MailStorageLookupSnafu {
                    user: user.to_string(),
                })?
            );
        } else {
            let output = cmd("doveadm", vec!["-f", "flow", "quota", "get", "-u", user]).context(
                MailStorageLookupSnafu {
                    user: user.to_string(),
                },
            )?;
            let output = String::from_utf8_lossy(&output.stdout);

            // Use a global Regex for the life of the program, in case we're running in a loop, because generating
            // the regex could become the bottleneck...
            // UNWRAP NOTE: The regex cannot fail because it's well-known.
            let re = REGEX_MAILUSAGE.get_or_init(|| Regex::new(r"Value=(\d+)").unwrap());
            if let Some(captures) = re.captures(&output) {
                // TODO: human format
                mail_use = captures.get(1).unwrap().as_str().to_string();
            }
        }

        Ok(Self { limit, mail_use })
    }
}

/// A permission on the Yunohost system, with associated users.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YunohostPermissionInfo {
    pub name: String,
    pub corresponding_users: Vec<String>,
}

impl YunohostPermissionInfo {
    pub fn permission(name: &str) -> Result<YunohostPermissionInfo, Error> {
        let permissions = LdapPermission::one_off_permissions()?;

        for perm in permissions {
            if perm.name == name {
                return Ok(YunohostPermissionInfo::try_from(perm)?);
            }
        }

        Err(Error::LdapPermissionNotFound {
            name: name.to_string(),
        })
    }
}

impl TryFrom<LdapPermission> for YunohostPermissionInfo {
    type Error = Error;

    fn try_from(perm: LdapPermission) -> Result<YunohostPermissionInfo, Error> {
        Ok(YunohostPermissionInfo {
            name: perm.name,
            // inheritPermission was requested so this should be safe unwrap
            corresponding_users: perm
                .attrs
                .get("inheritPermission")
                .unwrap()
                .into_iter()
                .map(|dn| LdapUser::uid_from_dn(&dn))
                .collect(),
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
pub struct YunohostUserInfo {
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

impl YunohostUserInfo {
    pub fn info_for<T: Into<UserQuery>>(query: T) -> Result<YunohostUserInfo, Error> {
        // Support name and mail
        let query: UserQuery = query.into();
        let user = LdapUser::one_off_get_user(&query)?;

        Ok(YunohostUserInfo::try_from(user)?)
    }
}

impl TryFrom<LdapUser> for YunohostUserInfo {
    type Error = Error;

    fn try_from(user: LdapUser) -> Result<YunohostUserInfo, Error> {
        debug!("{:#?}", &user);

        // let mailbox_quota = user.attrs.get("mailuserquota").map(|_x| "TODO".to_string());
        let mailbox_quota = MailStorageUse::from_name_and_quota(
            &user.name,
            user.attrs
                .get("mailuserquota")
                .unwrap()
                .iter()
                .nth(0)
                .unwrap(),
        )?;

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

        Ok(YunohostUserInfo {
            username: user.name,
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
            mailbox_quota,
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
