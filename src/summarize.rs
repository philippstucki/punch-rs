use chrono::{Duration, NaiveDate};
use itertools::Itertools;
use rusqlite::{Connection, NO_PARAMS};
use std::error::Error;
use std::result::Result;

use crate::colors::Colors;
use crate::datetime;

#[derive(Debug)]
struct SummaryRow {
    project_id: i64,
    group_day: NaiveDate,
    total_time: Duration,
    project_title: String,
}

fn group_rows_by_period(rows: Vec<SummaryRow>) -> Vec<(NaiveDate, Vec<SummaryRow>)> {
    rows.into_iter()
        .group_by(|row| row.group_day.clone())
        .into_iter()
        .map(|(day, day_rows)| (day, day_rows.collect()))
        .collect()
}

pub fn get_tag_summary_for_project(
    conn: &mut Connection,
    project_id: i64,
    period: NaiveDate,
) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            tag.title,
            CAST(
                total((strftime('%J',stopped_on)-strftime('%J',started_on))*24*3600)
                AS INTEGER
            )  AS duration,
            group_concat(timeslice_id)
        FROM tag
        LEFT JOIN timeslice_tag USING(tag_id)
        LEFT JOIN timeslice USING(timeslice_id)
        WHERE
            stopped_on IS NOT NULL
            AND strftime('%Y-%m-%d',stopped_on) = '2020-10-06'
            AND timeslice.project_id=13
            AND tag.title IS NOT NULL
        GROUP BY tag.tag_id
    ",
    )?;
    Ok(())
}

pub fn summarize_command(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            timeslice.project_id ,
            strftime('%Y-%m-%d',stopped_on) group_period,
            project.title,
            CAST(
                total((strftime('%J',stopped_on)-strftime('%J',started_on))*24*3600)
                AS INTEGER
            )  AS duration,
            max(stopped_on) AS row_order
        FROM timeslice
        JOIN project USING(project_id)
        WHERE stopped_on IS NOT NULL
        GROUP BY group_period, timeslice.project_id
        ORDER BY group_period, row_order ASC
    ",
    )?;

    let rows = stmt
        .query_map(NO_PARAMS, |row| {
            Ok(SummaryRow {
                project_id: row.get(0)?,
                group_day: NaiveDate::parse_from_str(&*row.get::<_, String>(1)?, "%Y-%m-%d")
                    .unwrap(),
                project_title: row.get::<_, String>(2)?,
                total_time: Duration::seconds(row.get(3)?),
            })
        })?
        .map(|row| row.unwrap())
        .collect::<Vec<SummaryRow>>();

    for (period, rows) in group_rows_by_period(rows) {
        println!(
            "{group_day}",
            group_day = period.format("%Y-%m-%d").to_string().color_heading()
        );
        for row in rows {
            println!(
                "    {project_title:<20} {duration:>14}",
                project_title = row.project_title.to_string().color_project(),
                duration = datetime::duration_as_hms_string(&row.total_time)?
                    .to_string()
                    .color_duration()
            );
        }
        println!("\n");
    }

    Ok(())
}
