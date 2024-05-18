use ldap3::dn_escape;
use serde::{Deserialize, Deserializer};
use snafu::prelude::*;

use std::str::FromStr;

use crate::error::*;

fn non_empty_string(s: &str) -> Option<String> {
    if s.trim() == "" {
        None
    } else {
        Some(s.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct Username(String);

impl Username {
    pub fn new(s: &str) -> Result<Username, Error> {
        non_empty_string(s)
            .context(LdapEmptyUsernameSnafu)
            .map(|s| Username(s))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn ldap_escape(&self) -> String {
        dn_escape(self.as_str()).to_string()
    }

    pub fn to_dn(&self, domains: &[&str]) -> String {
        let mut dn = format!("uid={},ou=users", self.ldap_escape());
        for domain in domains {
            dn.push_str(",dc=");
            dn.push_str(domain);
        }

        dn
    }
}

impl FromStr for Username {
    type Err = Error;

    fn from_str(s: &str) -> Result<Username, Error> {
        Username::new(s)
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "axum")]
#[async_trait]
impl TryFromChunks for Username {
    async fn try_from_chunks(
        chunks: impl Stream<Item = Result<Bytes, TypedMultipartError>> + Send + Sync + Unpin,
        metadata: FieldMetadata,
    ) -> Result<Self, TypedMultipartError> {
        let string = String::try_from_chunks(chunks, metadata).await?;
        let data =
            Self::from_str(&string).map_err(|e| TypedMultipartError::Other { source: e.into() })?;
        Ok(data)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Password(String);

impl Password {
    pub fn new(s: &str) -> Result<Password, Error> {
        non_empty_string(s)
            .context(LdapEmptyPasswordSnafu)
            .map(|s| Password(s))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn ldap_escape(&self) -> String {
        dn_escape(self.as_str()).to_string()
    }
}

impl FromStr for Password {
    type Err = Error;

    fn from_str(s: &str) -> Result<Password, Error> {
        Password::new(s)
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}
