use super::{encrypt_into_b64u, Error, Result};
use crate::config;
use crate::crypt::EncryptContent;

pub fn encrypt_password(enc_content: &EncryptContent) -> Result<String> {
    let key = &config().PASSWORD_KEY;
    let encrypted = encrypt_into_b64u(key, enc_content)?;

    Ok(format!("#01#{encrypted}"))
}

pub fn validate_password(encrypt_content: &EncryptContent, password: &str) -> Result<()> {
    let encrypted_password = encrypt_password(encrypt_content)?;

    if encrypted_password == password {
        Ok(())
    } else {
        Err(Error::PasswordMismatch)
    }
}
