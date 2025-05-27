use super::{Error, Result, Scheme};
use crate::config::auth_config;
use crate::pwd::ContentToHash;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use std::sync::OnceLock;

pub struct Scheme02;

impl Scheme for Scheme02 {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
        let argon2 = get_argon2();

        let salt_b64 = SaltString::encode_b64(to_hash.salt.as_bytes()).map_err(|_| Error::Salt)?;

        let pwd = argon2
            .hash_password(to_hash.content.as_bytes(), &salt_b64)
            .map_err(|_| Error::Hash)?
            .to_string();

        Ok(pwd)
    }

    fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()> {
        let argon2 = get_argon2();

        let parsed_hash = PasswordHash::new(pwd_ref).map_err(|_| Error::Hash)?;

        argon2
            .verify_password(to_hash.content.as_bytes(), &parsed_hash)
            .map_err(|_| Error::PwdValidate)
    }
}

fn get_argon2() -> &'static Argon2<'static> {
    static INSTANCE: OnceLock<Argon2<'static>> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        let key = &auth_config().PWD_KEY;

        Argon2::new_with_secret(
            key,
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::default(),
        )
        .unwrap() // TODO - needs to fail early
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pwd::ContentToHash;
    use anyhow::Result;
    use uuid::Uuid;

    #[test]
    fn test_scheme_02_hash_into_b64u_ok() -> Result<()> {
        let fx_key = &auth_config().PWD_KEY;
        let fx_to_hash = ContentToHash {
            content: "password".to_string(),
            salt: Uuid::parse_str("dccda68f-6def-44f4-b901-fabe784dc335")?,
        };
        let fx_res = "$argon2id$v=19$m=19456,t=2,p=1$3M2mj23vRPS5Afq+eE3DNQ$AtkdwkxWAmsXsQsOeJRM9BeTDQEG44zQbC+VV+VMl+s";

        let scheme = Scheme02;
        let res = scheme.hash(&fx_to_hash)?;
        assert_eq!(res, fx_res);

        Ok(())
    }
}
