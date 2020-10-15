use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Filter {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}
