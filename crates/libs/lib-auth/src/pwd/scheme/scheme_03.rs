use super::{Error, Result, Scheme};
use crate::config::auth_config;
use crate::pwd::ContentToHash;
use blake3::Hasher;

pub struct Scheme03;

impl Scheme for Scheme03 {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
        let key = &auth_config().PWD_KEY;
        let key: [u8; blake3::KEY_LEN] = *blake3::hash(key).as_bytes();

        // With update
        let mut hasher = Hasher::new_keyed(&key);
        hasher.update(to_hash.content.as_bytes());
        hasher.update(to_hash.salt.as_bytes());

        Ok(hasher.finalize().to_string())
    }

    fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: &str) -> Result<()> {
        if self.hash(to_hash)? == raw_pwd_ref {
            Ok(())
        } else {
            Err(Error::PwdValidate)
        }
    }
}
