use crate::config::auth_config;
use crate::pwd::ContentToHash;
use crate::pwd::scheme::{Error, Result, Scheme};
use hmac::{Hmac, Mac};
use lib_utils::b64::b64u_encode;
use sha2::Sha512;

pub struct Scheme01;

impl Scheme for Scheme01 {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
        let key = &auth_config().PWD_KEY;
        hash(key, to_hash)
    }

    fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: &str) -> Result<()> {
        let raw_pwd_new = self.hash(to_hash)?;
        if raw_pwd_new == raw_pwd_ref {
            Ok(())
        } else {
            Err(Error::PwdValidate)
        }
    }
}

pub fn hash(key: &[u8], to_hash: &ContentToHash) -> Result<String> {
    let ContentToHash { content, salt } = to_hash;

    // -- Create a HMAC-SHA-512 from key.
    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::Key)?;

    // -- Add content.
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // -- Finalize and b64u encode.
    let hmac_result = hmac_sha512.finalize();

    let result = b64u_encode(hmac_result.into_bytes());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pwd::ContentToHash;
    use anyhow::Result;
    use uuid::Uuid;

    #[test]
    fn test_scheme_01_hash_into_b64u_ok() -> Result<()> {
        let fx_salt = Uuid::parse_str("dccda68f-6def-44f4-b901-fabe784dc335")?;
        let fx_key = &auth_config().PWD_KEY;
        let fx_to_hash = ContentToHash {
            content: "password".to_string(),
            salt: fx_salt,
        };
        let fx_res = "QIE3zgC9ji7_3cAbSdAJEJ44EiZMD7ROgqpolT36DMcmm_CDs5nQGtI6pDmIKxOS0oEBRRwo0h1k_Q-jFF90uQ";

        let res = hash(fx_key, &fx_to_hash)?;
        assert_eq!(res, fx_res);

        Ok(())
    }
}
