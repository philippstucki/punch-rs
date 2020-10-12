use chrono::{Duration, NaiveDate};
use itertools::Itertools;
use rusqlite::{named_params, Connection, NO_PARAMS};
use std::error::Error;
use std::result::Result;

use crate::colors::Colors;
use crate::datetime;

fn group_rows_by_period(rows: Vec<PeriodSummaryRow>) -> Vec<(NaiveDate, Vec<PeriodSummaryRow>)> {
    rows.into_iter()
        .group_by(|row| row.period.clone())
        .into_iter()
        .map(|(period, period_rows)| (period, period_rows.collect()))
        .collect()
}

#[derive(Debug)]
struct TagSummary {
    tag_title: String,
    total_time: Duration,
}

fn get_tag_summary(
    conn: &Connection,
    project_id: i64,
    period: NaiveDate,
) -> Result<Vec<TagSummary>, Box<dyn Error>> {
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
            AND strftime('%Y-%m-%d',stopped_on) = :period
            AND timeslice.project_id = :project_id
            AND tag.title IS NOT NULL
        GROUP BY tag.tag_id
    ",
    )?;

    let rows = stmt
        .query_map_named(
            named_params! {
                ":project_id": project_id,
                ":period": period.format("%Y-%m-%d").to_string(),
            },
            |row| {
                Ok(TagSummary {
                    tag_title: row.get::<_, String>(0)?,
                    total_time: Duration::seconds(row.get(1)?),
                })
            },
        )?
        .map(|row| row.unwrap())
        .collect();
    Ok(rows)
}

#[derive(Debug)]
struct PeriodSummaryRow {
    project_id: i64,
    period: NaiveDate,
    total_time: Duration,
    project_title: String,
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
            Ok(PeriodSummaryRow {
                project_id: row.get(0)?,
                period: NaiveDate::parse_from_str(&*row.get::<_, String>(1)?, "%Y-%m-%d").unwrap(),
                project_title: row.get::<_, String>(2)?,
                total_time: Duration::seconds(row.get(3)?),
            })
        })?
        .map(|row| row.unwrap())
        .collect::<Vec<PeriodSummaryRow>>();

    for (period, rows) in group_rows_by_period(rows) {
        println!(
            "\n{period}",
            period = period.format("%Y-%m-%d").to_string().color_heading()
        );
        for row in rows {
            println!(
                "    {project_title:<20} {duration:>14}",
                project_title = row.project_title.to_string().color_project(),
                duration = datetime::duration_as_hms_string(&row.total_time)?
                    .to_string()
                    .color_duration()
            );

            let tag_summary = get_tag_summary(conn, row.project_id, row.period)?;
            for tag in &tag_summary {
                println!(
                    "      {tag_title:<18} {duration:>14}",
                    tag_title = tag.tag_title.color_tag(),
                    duration = datetime::duration_as_hms_string(&tag.total_time)?.color_duration()
                )
            }
            if tag_summary.len() > 0 {
                println!("\n");
            }
        }
    }

    Ok(())
}
