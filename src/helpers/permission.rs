use ldap3::{Scope, SearchEntry};
// use snafu::prelude::*;
use tokio::runtime::Builder as RuntimeBuilder;

use crate::{
    error::*,
    helpers::{ldap::*, user::*},
};

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
