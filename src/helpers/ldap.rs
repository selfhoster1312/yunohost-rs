use ldap3::{exop::WhoAmI, Ldap, LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use snafu::prelude::*;
use tokio::sync::RwLock;

use std::sync::Arc;
use std::time::Duration;

use crate::error::*;
use crate::helpers::credentials::*;

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
            error!("Failed LDAP query for {}, returning empty userlist.", query);
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
