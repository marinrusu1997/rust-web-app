mod error;
mod scheme;

pub use self::error::{Error, Result};
pub use scheme::SchemeStatus;

use crate::pwd::scheme::Scheme;
use lazy_regex::regex_captures;
use scheme::{DEFAULT_SCHEME, get_scheme};
use std::str::FromStr;
use uuid::Uuid;

pub struct ContentToHash {
    pub content: String, // Clear content.
    pub salt: Uuid,      // Clear salt.
}

pub async fn hash_pwd(to_hash: ContentToHash) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_for_scheme(DEFAULT_SCHEME, &to_hash))
        .await
        .map_err(|_| Error::FailSpawnBlockForHash)?
}

pub async fn validate_pwd(to_hash: ContentToHash, pwd_ref: &str) -> Result<SchemeStatus> {
    let PwdParts { scheme_name, hash } = pwd_ref.parse()?;
    let scheme_status = if scheme_name == DEFAULT_SCHEME {
        SchemeStatus::UpToDate
    } else {
        SchemeStatus::Outdated
    };

    tokio::task::spawn_blocking(move || validate_for_scheme(&scheme_name, &to_hash, &hash))
        .await
        .map_err(|_| Error::FailSpawnBlockForValidate)??;

    Ok(scheme_status)
}

fn hash_for_scheme(scheme_name: &str, to_hash: &ContentToHash) -> Result<String> {
    let scheme = get_scheme(scheme_name)?;
    let pwd_hashed = scheme.hash(to_hash)?;
    Ok(format!("#{scheme_name}#{pwd_hashed}"))
}

fn validate_for_scheme(scheme_name: &str, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()> {
    let scheme = get_scheme(scheme_name)?;
    scheme.validate(to_hash, pwd_ref)?;
    Ok(())
}

struct PwdParts {
    scheme_name: String,
    hash: String,
}

impl FromStr for PwdParts {
    type Err = Error;

    fn from_str(password_with_scheme: &str) -> Result<Self> {
        regex_captures!(r#"^#(\w+)#(.*)"#, password_with_scheme)
            .map(|(_, scheme, hashed)| Self {
                scheme_name: scheme.to_string(),
                hash: hashed.to_string(),
            })
            .ok_or(Error::PwdWithSchemeFailedToParse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_multi_scheme_ok() -> Result<()> {
        let fx_salt = Uuid::parse_str("dccda68f-6def-44f4-b901-fabe784dc335")?;
        let fx_to_hash = ContentToHash {
            content: "password".to_string(),
            salt: fx_salt,
        };

        let hashed_password = hash_for_scheme("01", &fx_to_hash)?;
        let pwd_validate = validate_pwd(fx_to_hash, &hashed_password).await?;

        assert!(matches!(pwd_validate, SchemeStatus::Outdated));

        Ok(())
    }
}
