use chrono::{Duration, NaiveDate};
use itertools::Itertools;
use rusqlite::{named_params, types::Value, Connection, NO_PARAMS};
use std::error::Error;
use std::rc::Rc;
use std::result::Result;

use crate::colors::Colors;
use crate::datetime;

pub enum GroupingMode {
    Day,
    All,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum PeriodSummaryGrouping {
    Day(NaiveDate),
    All,
}

#[derive(Debug)]
struct TagSummary {
    tag_title: String,
    total_time: Duration,
}

fn get_tag_summary(
    conn: &Connection,
    slice_ids: Vec<i64>,
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
            timeslice_id IN rarray(:slice_ids)
            AND tag.title IS NOT NULL
        GROUP BY tag.tag_id
    ",
    )?;

    let slice_ids = Rc::new(
        slice_ids
            .iter()
            .copied()
            .map(Value::from)
            .collect::<Vec<Value>>(),
    );
    let rows = stmt
        .query_map_named(
            named_params! {
                ":slice_ids": slice_ids
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
    grouping: PeriodSummaryGrouping,
    total_time: Duration,
    project_title: String,
    slice_ids: Vec<i64>,
}

fn group_summary_rows(
    rows: Vec<PeriodSummaryRow>,
) -> Vec<(PeriodSummaryGrouping, Vec<PeriodSummaryRow>)> {
    rows.into_iter()
        .group_by(|row| row.grouping)
        .into_iter()
        .map(|(group, group_rows)| (group, group_rows.collect()))
        .collect()
}

fn parse_group_concat(from: String) -> Vec<i64> {
    from.split(",")
        .into_iter()
        .map(|v| v.parse::<i64>().unwrap())
        .collect()
}

pub fn summarize_command(
    conn: &mut Connection,
    grouping_mode: GroupingMode,
) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare(&format!(
        "
        SELECT
            timeslice.project_id,
            strftime('{period}', stopped_on) group_period,
            project.title,
            CAST(
                total((strftime('%J',stopped_on) - strftime('%J',started_on)) * 24 * 3600)
                AS INTEGER
            ) AS duration,
            max(stopped_on) AS row_order,
            GROUP_CONCAT(timeslice.timeslice_id)
        FROM timeslice
        JOIN project USING(project_id)
        WHERE stopped_on IS NOT NULL
        GROUP BY group_period, timeslice.project_id
        ORDER BY group_period, row_order DESC
    ",
        period = if let GroupingMode::Day = grouping_mode {
            "%Y-%m-%d"
        } else {
            "ALL"
        }
    ))?;

    let rows = stmt
        .query_map(NO_PARAMS, |row| {
            let grouping = match grouping_mode {
                GroupingMode::Day => PeriodSummaryGrouping::Day(datetime::naivedate_from_string(
                    &*row.get::<_, String>(1)?,
                )),
                _ => PeriodSummaryGrouping::All,
            };

            Ok(PeriodSummaryRow {
                project_id: row.get(0)?,
                grouping,
                project_title: row.get::<_, String>(2)?,
                total_time: Duration::seconds(row.get(3)?),
                slice_ids: parse_group_concat(row.get::<_, String>(5)?),
            })
        })?
        .map(|row| row.unwrap())
        .collect::<Vec<PeriodSummaryRow>>();

    for (grouping, rows) in group_summary_rows(rows) {
        if let PeriodSummaryGrouping::Day(date) = grouping {
            println!(
                "\n{grouping}",
                grouping = datetime::naivedate_format(date).color_heading()
            );
        }
        for row in rows {
            println!(
                "    {project_title:<20} {duration:>14}",
                project_title = row.project_title.to_string().color_project(),
                duration = datetime::duration_as_hms_string(&row.total_time)?
                    .to_string()
                    .color_duration()
            );

            let tag_summary = get_tag_summary(conn, row.slice_ids)?;
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
