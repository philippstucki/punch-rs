use chrono::{DateTime, Local, Utc};
use rusqlite::{Connection, OptionalExtension, NO_PARAMS};
use std::error::Error;
use std::result::Result;

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
            started_on: datetime::as_local(datetime::from_string(started_on)),
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

pub fn start_command(conn: &mut Connection, project_name: &str) -> Result<(), Box<dyn Error>> {
    match get_running_slice(conn)? {
        None => {
            let tx = conn.transaction()?;
            let project_id = match db::project_get_by_name(&tx, project_name)? {
                Some(project) => project.id,
                None => db::project_create(&tx, project_name)?,
            };
            db::timeslice_create(
                &tx,
                db::Timeslice {
                    id: None,
                    project_id,
                    started_on: Utc::now(),
                    stopped_on: None,
                },
            )?;
            tx.commit()?;
        }
        Some(slice) => println!("found running slice: {:?}", slice),
    };
    Ok(())
}
