use chrono::{Duration, NaiveDate};
use itertools::Itertools;
use rusqlite::{Connection, NO_PARAMS};
use std::error::Error;
use std::result::Result;

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

pub fn summary_command(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            t.project_id,
            strftime(\"%Y-%m-%d\",stopped_on) group_period,
            p.title,
            CAST(
                total((strftime(\"%J\",stopped_on)-strftime(\"%J\",started_on))*24*3600)
                AS INTEGER
            )  AS duration,
            max(stopped_on) AS row_order
        FROM timeslice t
        JOIN project p USING(project_id)
        GROUP BY group_period, t.project_id
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
        println!("{group_day}", group_day = period.format("%Y-%m-%d"));
        for row in rows {
            println!(
                "    {project_title:<20} {duration:>14}",
                project_title = row.project_title,
                duration = datetime::duration_as_hms_string(&row.total_time)?
            );
        }
        println!("\n");
    }

    Ok(())
}
