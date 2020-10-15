use chrono::{DateTime, Duration, Local, NaiveDate};
use itertools::Itertools;
use rusqlite::{named_params, Connection};
use std::error::Error;
use std::result::Result;

use crate::colors::Colors;
use crate::datetime;
use crate::filter::Filter;

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
    day: NaiveDate,
    started_on: DateTime<Local>,
    stopped_on: DateTime<Local>,
    duration: Duration,
    project_name: String,
    tags: Vec<String>,
}

impl LogTimeslice {
    fn new(
        id: i64,
        day: &str,
        started_on: &str,
        stopped_on: &str,
        project_name: &str,
        tags: &str,
    ) -> LogTimeslice {
        let started_on = datetime::as_local(datetime::from_rfc3339_string(started_on));
        let stopped_on = datetime::as_local(datetime::from_rfc3339_string(stopped_on));

        LogTimeslice {
            id,
            day: datetime::naivedate_from_string(day),
            started_on,
            stopped_on,
            duration: stopped_on - started_on,
            project_name: String::from(project_name),
            tags: if tags.len() > 0 {
                tags.split(",").map(|tag| tag.to_string()).collect()
            } else {
                vec![]
            },
        }
    }
}

fn group_slices_by_day(slices: Vec<LogTimeslice>) -> Vec<(NaiveDate, Vec<LogTimeslice>)> {
    slices
        .into_iter()
        .group_by(|r| r.day.clone())
        .into_iter()
        .map(|(day, day_slices)| (day, day_slices.collect()))
        .collect()
}

pub fn log_command(conn: &mut Connection, filter: &Filter) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            timeslice_id,
            date(stopped_on) day,
            started_on,
            stopped_on,
            project.title,
            COALESCE(GROUP_CONCAT(tag.title), '')
        FROM timeslice
        JOIN project USING(project_id)
        LEFT JOIN timeslice_tag USING(timeslice_id)
		LEFT JOIN tag USING(tag_id)
        WHERE
            stopped_on IS NOT NULL
            AND stopped_on >= :filter_from_date
        GROUP BY timeslice_id
        ORDER BY day ASC
    ",
    )?;

    let from_date = filter
        .from
        .unwrap_or(datetime::timestamp_1970())
        .format(datetime::DATE_FORMAT_YMD)
        .to_string();

    let slices = stmt
        .query_map_named(named_params! {":filter_from_date": from_date}, |row| {
            Ok(LogTimeslice::new(
                row.get(0)?,
                &*row.get::<_, String>(1)?,
                &*row.get::<_, String>(2)?,
                &*row.get::<_, String>(3)?,
                &*row.get::<_, String>(4)?,
                &*row.get::<_, String>(5)?,
            ))
        })?
        .map(|r| r.unwrap())
        .collect::<Vec<LogTimeslice>>();

    for (day, slices) in group_slices_by_day(slices) {
        println!("{}\n", datetime::naivedate_format(day).color_heading());

        for slice in slices {
            let tags = match slice.tags.len() > 0 {
                true => format!("({})", slice.tags.join(", ").color_tag()),
                false => String::from(""),
            };
            println!(
                "    {started_on} — {stopped_on} {duration:>14} {project_name} {tags}",
                started_on = slice.started_on.format("%H:%M:%S").to_string().color_time(),
                stopped_on = slice.stopped_on.format("%H:%M:%S").to_string().color_time(),
                duration = datetime::duration_as_hms_string(&slice.duration)?
                    .to_string()
                    .color_duration(),
                project_name = slice.project_name.to_string().color_project(),
                tags = tags
            );
        }

        println!("\n")
    }

    Ok(())
}
