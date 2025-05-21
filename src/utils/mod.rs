use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

mod error;

pub use error::{Error, Result};

pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

pub fn format_time(time: OffsetDateTime) -> String {
    time.format(&Rfc3339).unwrap() // TODO: need to check if safe
}

pub fn now_utc_plus_sec_str(sec: f64) -> String {
    format_time(now_utc() + Duration::seconds_f64(sec))
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(moment, &Rfc3339).map_err(|_| Error::DateFailParse(moment.to_string()))
}

pub fn b64u_encode(data: &str) -> String {
    base64_url::encode(data)
}

pub fn b64u_decode(data: &str) -> Result<String> {
    let decoded_data = base64_url::decode(data)
        .ok()
        .and_then(|v| String::from_utf8(v).ok())
        .ok_or(Error::FailToB64uDecode)?;

    Ok(decoded_data)
}
