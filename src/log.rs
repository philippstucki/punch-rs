use chrono::{DateTime, Duration, FixedOffset};
use itertools::Itertools;
use rusqlite::{Connection, NO_PARAMS};
use std::error::Error;

// use crate::db;

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
    started_on: DateTime<FixedOffset>,
    stopped_on: DateTime<FixedOffset>,
    duration: Duration,
    project_name: String,
}

fn group_slices_by_day(slices: Vec<LogTimeslice>) -> Vec<(String, Vec<LogTimeslice>)> {
    slices
        .into_iter()
        .group_by(|r| r.day.clone())
        .into_iter()
        .map(|(day, day_slices)| (day, day_slices.collect::<Vec<LogTimeslice>>()))
        .collect()
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
        ORDER BY day desc
        LIMIT 20
    ",
    )?;

    let slices = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(LogTimeslice {
                id: row.get(0)?,
                day: row.get(1)?,
                started_on: DateTime::parse_from_rfc3339(&*row.get::<_, String>(2)?).unwrap(),
                stopped_on: DateTime::parse_from_rfc3339(&*row.get::<_, String>(3)?).unwrap(),
                duration: Duration::seconds(0),
                project_name: row.get(4)?,
            })
        })?
        .map(|r| r.unwrap())
        .collect::<Vec<LogTimeslice>>();

    for (day, slices) in group_slices_by_day(slices) {
        println!("{}\n", day);

        for slice in slices {
            println!(
                "    {} — {}",
                slice.started_on.format("%H:%M"),
                slice.stopped_on.format("%H:%M")
            );
        }

        println!("\n")
    }

    Ok(())
}