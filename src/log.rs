use chrono::{DateTime, Duration, Local};
use itertools::Itertools;
use rusqlite::{Connection, NO_PARAMS};
use std::error::Error;
use std::fmt::Write;
use std::result::Result;

use crate::datetime;

/*
# output format:

<date>:
    <from> — <to>    <duration>  <project>  (tag_1, ..., tag_k)

# example output:

2020-09-12
    08:20 — 12:05       3h 45m  website (backend, admin)
    15:26 — 18:10       2h 44m  website (frontend)

*/

#[derive(Debug)]
struct LogTimeslice {
    id: i64,
    day: String,
    started_on: DateTime<Local>,
    stopped_on: DateTime<Local>,
    duration: Duration,
    project_name: String,
}

impl LogTimeslice {
    fn new(
        id: i64,
        day: &str,
        started_on: &str,
        stopped_on: &str,
        project_name: &str,
    ) -> LogTimeslice {
        let started_on = datetime::as_local(datetime::from_rfc3339_string(started_on));
        let stopped_on = datetime::as_local(datetime::from_rfc3339_string(stopped_on));

        LogTimeslice {
            id,
            day: String::from(day),
            started_on,
            stopped_on,
            duration: stopped_on - started_on,
            project_name: String::from(project_name),
        }
    }
}

fn group_slices_by_day(slices: Vec<LogTimeslice>) -> Vec<(String, Vec<LogTimeslice>)> {
    slices
        .into_iter()
        .group_by(|r| r.day.clone())
        .into_iter()
        .map(|(day, day_slices)| (day, day_slices.collect()))
        .collect()
}

fn duration_as_hms_string(duration: &Duration) -> Result<String, Box<dyn Error>> {
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

pub fn log_command(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            t.timeslice_id,
            date(t.stopped_on) day,
            t.started_on,
            t.stopped_on,
            p.title
        FROM timeslice t
        JOIN project p USING(project_id)
        WHERE stopped_on IS NOT NULL
        ORDER BY day desc
    ",
    )?;

    let slices = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(LogTimeslice::new(
                row.get(0)?,
                &*row.get::<_, String>(1)?,
                &*row.get::<_, String>(2)?,
                &*row.get::<_, String>(3)?,
                &*row.get::<_, String>(4)?,
            ))
        })?
        .map(|r| r.unwrap())
        .collect::<Vec<LogTimeslice>>();

    for (day, slices) in group_slices_by_day(slices) {
        println!("{}\n", day);

        for slice in slices {
            println!(
                "    {started_on} — {stopped_on} {duration:>14} {project_name}",
                started_on = slice.started_on.format("%H:%M:%S"),
                stopped_on = slice.stopped_on.format("%H:%M:%S"),
                duration = duration_as_hms_string(&slice.duration)?,
                project_name = slice.project_name
            );
        }

        println!("\n")
    }

    Ok(())
}
