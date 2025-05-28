mod error;
mod scheme_01;
mod scheme_02;

pub use self::error::{Error, Result};
use crate::pwd::ContentToHash;
use enum_dispatch::enum_dispatch;

pub const DEFAULT_SCHEME: &str = "02";

#[derive(Debug)]
pub enum SchemeStatus {
    UpToDate,
    Outdated,
}

#[enum_dispatch]
pub trait Scheme {
    fn hash(&self, to_hash: &ContentToHash) -> Result<String>;
    fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()>;
}

#[enum_dispatch(Scheme)]
enum SchemeDispatcher {
    Scheme01(scheme_01::Scheme01),
    Scheme02(scheme_02::Scheme02),
}

pub fn get_scheme(scheme: &str) -> Result<impl Scheme> {
    match scheme {
        "01" => Ok(SchemeDispatcher::Scheme01(scheme_01::Scheme01)),
        "02" => Ok(SchemeDispatcher::Scheme02(scheme_02::Scheme02)),
        _ => Err(Error::SchemeNotFound(scheme.to_string())),
    }
}
