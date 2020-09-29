use std::fmt::Display;

use chrono::{DateTime, FixedOffset, Local, TimeZone, Utc};

// #[derive(Debug)]
// pub struct DateTime<T: chrono::TimeZone> {
//     datetime: chrono::DateTime<T>,
// }

// impl<T: chrono::TimeZone> From<&str> for DateTime<T> {
//     fn from(as_string: &str) -> Self {
//         DateTime {
//             datetime: chrono::DateTime::from(
//                 chrono::DateTime::parse_from_rfc3339(as_string).unwrap(),
//             ),
//         }
//     }
// }

// impl From<&str> for chrono::DateTime

pub fn from_string(as_string: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(as_string).unwrap()
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
