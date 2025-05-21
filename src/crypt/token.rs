use super::error::{Error, Result};
use crate::config;
use crate::crypt::{encrypt_into_b64u, EncryptContent};
use crate::utils::{b64u_decode, b64u_encode, now_utc, now_utc_plus_sec_str, parse_utc};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub struct Token {
    pub iden: String,      // Identifier (username for example)
    pub exp: String,       // Expiration time in RFC3339 format
    pub sign_b64u: String, // Base64 URL encoded signature
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            b64u_encode(&self.iden),
            b64u_encode(&self.exp),
            self.sign_b64u
        )
    }
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(token_str: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = token_str.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::TokenInvalidFormat);
        }
        let (iden_b64u, exp_b64u, sign_b64u) = (parts[0], parts[1], parts[2]);

        Ok(Self {
            iden: b64u_decode(iden_b64u).map_err(|_| Error::TokenCannotDecodeIdentity)?,
            exp: b64u_decode(exp_b64u).map_err(|_| Error::TokenCannotDecodeExpiration)?,
            sign_b64u: sign_b64u.to_string(),
        })
    }
}

pub fn generate_web_token(username: &str, salt: &str) -> Result<Token> {
    let config = config();
    generate_token(username, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(token: &Token, salt: &str) -> Result<()> {
    let config = config();
    validate_token_sign_and_exp(token, salt, &config.TOKEN_KEY)?;
    Ok(())
}

fn generate_token(iden: &str, duration_sec: f64, salt: &str, key: &[u8]) -> Result<Token> {
    let iden = iden.to_string();
    let exp = now_utc_plus_sec_str(duration_sec);
    let sign_b64u = token_sign_into_b64u(&iden, &exp, salt, key)?;

    Ok(Token {
        iden,
        exp,
        sign_b64u,
    })
}

fn validate_token_sign_and_exp(token: &Token, salt: &str, key: &[u8]) -> Result<()> {
    let new_sign_b64u = token_sign_into_b64u(&token.iden, &token.exp, salt, key)?;
    if new_sign_b64u != token.sign_b64u {
        return Err(Error::TokenSignatureNotMatching);
    }

    let token_exp = parse_utc(&token.exp).map_err(|_| Error::TokenExpirationNotIso)?;
    let now = now_utc();
    if token_exp < now {
        return Err(Error::TokenExpired);
    }

    Ok(())
}

fn token_sign_into_b64u(iden: &str, exp: &str, salt: &str, key: &[u8]) -> Result<String> {
    let content = format!("{}.{}", b64u_encode(iden), b64u_encode(exp));
    let signature = encrypt_into_b64u(
        key,
        &EncryptContent {
            content,
            salt: salt.to_string(),
        },
    )?;

    Ok(signature)
}

#[cfg(test)]
mod tests {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For early dev & tests.
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_token_display_ok() -> Result<()> {
        // -- Fixtures
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyMy0wNS0xN1QxNTozMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            iden: "fx-ident-01".to_string(),
            exp: "2023-05-17T15:30:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // -- Exec & Check
        assert_eq!(fx_token.to_string(), fx_token_str);

        Ok(())
    }

    #[test]
    fn test_token_from_str_ok() -> Result<()> {
        // -- Fixtures
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyMy0wNS0xN1QxNTozMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            iden: "fx-ident-01".to_string(),
            exp: "2023-05-17T15:30:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // -- Exec
        let token: Token = fx_token_str.parse()?;

        // -- Check
        assert_eq!(format!("{token:?}"), format!("{fx_token:?}"));

        Ok(())
    }

    #[test]
    fn test_token_validate_web_token_ok() -> Result<()> {
        // -- Setup & Fixtures
        let fx_user = "user_one";
        let fx_salt = "f05e8961-d6ad-4086-9e78-a6de065e5453";
        let fx_duration_sec = 0.02; // 20ms
        let token_key = &config().TOKEN_KEY;
        let fx_token = generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // -- Exec
        thread::sleep(Duration::from_millis(10));
        let res = validate_web_token(&fx_token, fx_salt);

        // -- Check
        res?;

        Ok(())
    }

    #[test]
    fn test_token_validate_web_token_err_expired() -> Result<()> {
        // -- Setup & Fixtures
        let fx_user = "user_one";
        let fx_salt = "f05e8961-d6ad-4086-9e78-a6de065e5453";
        let fx_duration_sec = 0.01; // 10ms
        let token_key = &config().TOKEN_KEY;
        let fx_token = generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // -- Exec
        thread::sleep(Duration::from_millis(20));
        let res = validate_web_token(&fx_token, fx_salt);

        // -- Check
        assert!(
            matches!(res, Err(super::Error::TokenExpired)),
            "Should have matched `Err(Error::TokenExpired)` but was `{res:?}`"
        );

        Ok(())
    }
}
