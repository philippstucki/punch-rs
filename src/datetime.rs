use std::error::Error;
use std::fmt::Display;
use std::fmt::Write;

use chrono::{DateTime, Duration, FixedOffset, Local, TimeZone, Utc};

pub fn from_rfc3339_string(as_string: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(as_string).unwrap()
}

#[allow(dead_code)]
pub fn from_ymd_string(as_string: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_str(as_string, "%Y-%m-%d").unwrap()
}

pub fn as_local<T: TimeZone>(dt: DateTime<T>) -> DateTime<Local> {
    dt.with_timezone(&Local)
}

#[allow(dead_code)]
pub fn as_utc<T: TimeZone>(dt: DateTime<T>) -> DateTime<Utc> {
    dt.with_timezone(&Utc)
}

pub fn format_as_hms<T: TimeZone>(dt: DateTime<T>) -> String
where
    T::Offset: Display,
{
    format!("{}", dt.format("%H:%M:%S"))
}

pub fn duration_as_hms_string(duration: &Duration) -> Result<String, Box<dyn Error>> {
    let mut out = String::new();
    write!(
        out,
        "{:2}h {:2}m {:2}s",
        duration.num_hours(),
        duration.num_minutes() % 60,
        duration.num_seconds() % 60
    )?;
    Ok(out)
}
