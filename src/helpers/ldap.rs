use ldap3::{exop::WhoAmI, Ldap, LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use serde::Deserialize;
use snafu::prelude::*;
use tokio::{runtime::Builder as RuntimeBuilder, sync::RwLock};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::error::*;
use crate::helpers::{
    credentials::*,
    users::{UserAttr, UserQuery},
};

const LDAP_PATH: &'static str = "ldapi://%2Fvar%2Frun%2Fslapd%2Fldapi";

/// Opens a new LDAP connection. Does not guarantee it will stay alive...
/// Do some keepalive for that, such as [`YunohostLDAP::keepalive`].
async fn new_ldap(timeout: Duration) -> Result<Ldap, Error> {
    let settings = LdapConnSettings::new().set_conn_timeout(timeout);
    debug!(
        "Opening new LDAP connection with timeout: {}ms",
        timeout.as_millis()
    );

    let (conn, ldap) = LdapConnAsync::with_settings(settings, LDAP_PATH)
        .await
        .context(LdapInitSnafu {
            uri: LDAP_PATH.to_string(),
        })?;

    tokio::spawn(async move {
        if let Err(e) = conn.drive().await {
            error!("{}", e);
        }
    });

    Ok(ldap)
}

pub struct YunohostLDAP {
    timeout: Duration,
    inner: Arc<RwLock<Ldap>>,
}

impl YunohostLDAP {
    /// Open a new connection to Yunohost LDAP database with a specific timeout in milliseconds
    pub async fn new(timeout: u64) -> Result<Self, Error> {
        let timeout = Duration::from_millis(timeout);
        Ok(Self {
            timeout: timeout.clone(),
            inner: Arc::new(RwLock::new(new_ldap(timeout).await?)),
        })
    }

    pub async fn keepalive(&self) -> Result<(), Error> {
        {
            let mut ldap = self.inner.write().await;
            if !ldap.is_closed() {
                // check connection is still alive with WhoAmI
                ldap.with_timeout(self.timeout.clone());
                if ldap.extended(WhoAmI).await.is_ok() {
                    return Ok(());
                }
            }
        }
        log::warn!("LDAP connection has been closed. Opening again.");
        let mut ldap = self.inner.clone().write_owned().await;
        *ldap = new_ldap(self.timeout.clone()).await?;
        Ok(())
    }

    pub async fn check_credentials(
        &self,
        username: &Username,
        password: &Password,
    ) -> Result<bool, Error> {
        self.keepalive().await?;
        let mut ldap = self.inner.write().await;

        let username = username.to_dn(&["yunohost", "org"]);
        let password = password.ldap_escape();

        log::debug!("Attempting LDAP login for {}", &username);
        ldap.with_timeout(self.timeout.clone());
        let reply = ldap
            .simple_bind(&username, &password)
            .await
            .context(LdapBindSnafu)?;
        log::debug!("{:#?}", reply);

        // Successful auth if return code is 0
        Ok(reply.rc == 0)
    }

    pub async fn list<
        'a,
        S: AsRef<str> + Send + Sync + 'a,
        A: AsRef<[S]> + Send + Sync + std::fmt::Debug + 'a,
    >(
        &self,
        query: &str,
        scope: Scope,
        filter: &str,
        attrs: A,
    ) -> Result<Vec<SearchEntry>, Error> {
        self.keepalive().await?;
        let mut ldap = self.inner.write().await;
        ldap.with_timeout(self.timeout.clone());

        debug!("LDAP: Query list {query}");
        debug!("LDAP: Query list filter: {filter}");
        debug!("LDAP: Query list attrs: {attrs:?}");

        let res = ldap
            .search(query, scope, filter, attrs)
            .await
            .context(LdapSearchSnafu)?;

        if let Ok((res2, _)) = res.clone().success() {
            let res = res2
                .into_iter()
                .map(|x| SearchEntry::construct(x))
                .collect();
            Ok(res)
        } else {
            error!("Failed LDAP query for users, returning empty userlist.");
            Ok(Vec::new())
        }
    }

    pub async fn search<'a, S: AsRef<str> + Send + Sync + 'a, A: AsRef<[S]> + Send + Sync + 'a>(
        &self,
        query: &str,
        scope: Scope,
        filter: &str,
        attrs: A,
        err: Error,
    ) -> Result<SearchEntry, Error> {
        if let Some(entry) = self.search_option(query, scope, filter, attrs).await? {
            Ok(entry)
        } else {
            return Err(err);
        }
    }

    pub async fn search_option<
        'a,
        S: AsRef<str> + Send + Sync + 'a,
        A: AsRef<[S]> + Send + Sync + 'a,
    >(
        &self,
        query: &str,
        scope: Scope,
        filter: &str,
        attrs: A,
    ) -> Result<Option<SearchEntry>, Error> {
        self.keepalive().await?;
        let mut ldap = self.inner.write().await;
        ldap.with_timeout(self.timeout.clone());

        let res = ldap
            .search(query, scope, filter, attrs)
            .await
            .context(LdapSearchSnafu)?;

        if let Ok((res, _)) = res.success() {
            let res = res.into_iter().take(1).next().unwrap();
            Ok(Some(SearchEntry::construct(res)))
        } else {
            Ok(None)
        }
    }
}

/// Raw data about a user, from the LDAP database. Also contains one-off operation to find a user.
///
/// Use [`LdapUsers`] for one-off operations about multiple users.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdapUser {
    pub name: String,
    pub dn: String,
    pub attrs: HashMap<String, Vec<String>>,
    pub other: HashMap<String, Vec<Vec<u8>>>,
}

impl LdapUser {
    pub fn uid_from_dn(dn: &str) -> String {
        // TODO: error handling?
        // It should be safe that slapd returns correct results...
        dn.strip_prefix("uid=")
            .map(|x| {
                let mut split = x.split(',');
                split.next().unwrap().to_string()
            })
            .unwrap()
    }

    pub fn from_search_entry(se: SearchEntry) -> LdapUser {
        LdapUser {
            name: Self::uid_from_dn(&se.dn),
            dn: se.dn,
            attrs: se.attrs,
            other: se.bin_attrs,
        }
    }

    pub fn one_off_get_user(query: &UserQuery) -> Result<LdapUser, Error> {
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
                    // &user.to_dn(&["yunohost", "org"]),
                    // ou=users,dc=yunohost,dc=org",
                    &format!("{},ou=users,dc=yunohost,dc=org", query.to_ldap_filter()),
                    Scope::Base,
                    // "(objectclass=*)",
                    // &query.to_ldap_filter(),
                    "(objectclass=*)",
                    attrs,
                    Error::LdapNoSuchUser {
                        query: query.clone(),
                    },
                )
                .await?;

            Ok(user)
        })?;

        Ok(LdapUser::from_search_entry(user))
    }

    // TODO: custom attrs
    pub fn one_off_list_users<
        'a,
        S: AsRef<str> + Send + Sync + 'a,
        A: AsRef<[S]> + Send + Sync + 'a + std::fmt::Debug,
    >(
        attrs: A,
    ) -> Result<Vec<LdapUser>, Error> {
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
                    // "ou=users",
                    Scope::OneLevel,
                    "(&(objectclass=person)(!(uid=root))(!(uid=nobody)))",
                    attrs,
                )
                .await?;

            Ok(user_list)
        })?;

        let mut new_list: Vec<LdapUser> = vec![];
        for entry in user_list {
            new_list.push(LdapUser::from_search_entry(entry));
        }

        Ok(new_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_from_dn() {
        assert_eq!(
            LdapUser::uid_from_dn("uid=username,ou=users,dc=yunohost,dc=org"),
            String::from("username")
        )
    }
}
