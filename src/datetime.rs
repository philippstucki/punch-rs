use std::error::Error;
use std::fmt::Display;
use std::fmt::Write;

const DATE_FORMAT: &'static str = "%a %d %B %Y";

use chrono::{DateTime, Duration, FixedOffset, Local, NaiveDate, TimeZone, Utc};

pub fn from_rfc3339_string(as_string: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(as_string).unwrap()
}

pub fn naivedate_from_string(as_string: &str) -> NaiveDate {
    NaiveDate::parse_from_str(as_string, "%Y-%m-%d").unwrap()
}

pub fn naivedate_format(d: NaiveDate) -> String {
    format!("{}", d.format(DATE_FORMAT))
}

pub fn as_local<T: TimeZone>(dt: DateTime<T>) -> DateTime<Local> {
    dt.with_timezone(&Local)
}

#[allow(dead_code)]
pub fn as_utc<T: TimeZone>(dt: DateTime<T>) -> DateTime<Utc> {
    dt.with_timezone(&Utc)
}

pub fn datetime_as_time_string<T: TimeZone>(dt: &DateTime<T>) -> String
where
    T::Offset: Display,
{
    format!("{}", dt.format("%H:%M:%S"))
}

#[allow(dead_code)]
pub fn datetime_as_date_string<T: TimeZone>(dt: &DateTime<T>) -> String
where
    T::Offset: Display,
{
    format!("{}", dt.format("%A %F"))
}

#[allow(dead_code)]
pub fn datetime_as_datetime_string<T: TimeZone>(dt: DateTime<T>) -> String
where
    T::Offset: Display,
{
    format!(
        "{} {}",
        datetime_as_date_string(&dt),
        datetime_as_time_string(&dt)
    )
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
