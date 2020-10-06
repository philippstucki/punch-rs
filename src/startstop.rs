use chrono::{DateTime, Local, Utc};
use rusqlite::{params, Connection, OptionalExtension, NO_PARAMS};
use std::error::Error;
use std::result::Result;

use crate::colors::Colors;
use crate::datetime;
use crate::db;

#[derive(Debug)]
struct RunningTimeslice {
    id: i64,
    started_on: DateTime<Local>,
    project_name: String,
}

impl RunningTimeslice {
    fn new(id: i64, started_on: &str, project_name: &str) -> RunningTimeslice {
        RunningTimeslice {
            id,
            started_on: datetime::as_local(datetime::from_rfc3339_string(started_on)),
            project_name: String::from(project_name),
        }
    }
}

fn get_running_slice(conn: &Connection) -> Result<Option<RunningTimeslice>, Box<dyn Error>> {
    match conn
        .query_row::<RunningTimeslice, _, _>(
            "SELECT t.timeslice_id, t.started_on, p.title FROM timeslice t JOIN project p USING(project_id) WHERE t.stopped_on IS NULL",
            NO_PARAMS,
            |row| Ok(RunningTimeslice::new(
                 row.get(0)?,
                 &*row.get::<_, String>(1)?,
                 &*row.get::<_, String>(2)?))
        )
        .optional()? {
        Some(slice) => Ok(Some(slice)),
        _ => Ok(None),
    }
}

pub fn start_command(
    conn: &mut Connection,
    project_name: &str,
    tags: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    match get_running_slice(conn)? {
        None => {
            let tx = conn.transaction()?;
            let project_id = match db::project_get_by_name(&tx, project_name)? {
                Some(project) => project.id,
                None => db::project_create(&tx, project_name)?,
            };

            let timeslice_id = db::timeslice_create(
                &tx,
                db::Timeslice {
                    id: None,
                    project_id,
                    started_on: Utc::now(),
                    stopped_on: None,
                },
            )?;

            for tag in tags.clone().into_iter() {
                let tag_id = db::tag_get_id_or_create(
                    &tx,
                    db::TagCreate {
                        project_id,
                        title: tag.to_string(),
                    },
                )?;

                db::tag_assign_to_timeslice(
                    &tx,
                    db::TimesliceTagCreate {
                        tag_id,
                        timeslice_id,
                    },
                )?;
            }
            println!(
                "started project {} with tags {}",
                project_name.color_project(),
                tags.join(" ").color_tag()
            );
            tx.commit()?;
        }
        Some(slice) => println!(
            "Slice already running for project {} started on {}",
            slice.project_name,
            datetime::format_as_hms(slice.started_on)
        ),
    };
    Ok(())
}

pub fn stop_command(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    match get_running_slice(conn)? {
        Some(slice) => {
            conn.execute(
                "UPDATE timeslice SET stopped_on = ?1 WHERE timeslice_id = ?2",
                params![Utc::now(), slice.id],
            )?;
        }
        None => println!("No running slice found."),
    };
    Ok(())
}
