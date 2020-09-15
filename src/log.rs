use chrono::{DateTime, Duration, FixedOffset, Utc};
use rusqlite::Connection;
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
    started_on: DateTime<FixedOffset>,
    stopped_on: DateTime<FixedOffset>,
    duration: Duration,
    project_name: String,
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
    ",
    )?;

    let row_iter = stmt.query_map_named(&[], |row| {
        Ok(LogTimeslice {
            id: row.get(0)?,
            started_on: DateTime::parse_from_rfc3339(&*row.get::<_, String>(2)?).unwrap(),
            stopped_on: DateTime::parse_from_rfc3339(&*row.get::<_, String>(3)?).unwrap(),
            duration: Duration::seconds(0),
            project_name: row.get(4)?,
        })
    })?;

    for row in row_iter {
        println!("{:?}", row);
    }

    // let person_iter = stmt.query_map(params![], |row| {
    //     Ok(Person {
    //         id: row.get(0)?,
    //         name: row.get(1)?,
    //         data: row.get(2)?,
    //     })
    // })?;

    Ok(())
}
