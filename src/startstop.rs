use chrono::{DateTime, Local};
use rusqlite::{Connection, OptionalExtension, NO_PARAMS};
use std::error::Error;
use std::result::Result;

#[derive(Debug)]
struct RunningTimeslice {
    id: i64,
    started_on: DateTime<Local>,
    project_name: String,
}

impl RunningTimeslice {
    fn new(id: i64, started_on: &str, project_name: &str) -> RunningTimeslice {
        let started_on: DateTime<Local> =
            DateTime::from(DateTime::parse_from_rfc3339(started_on).unwrap());
        RunningTimeslice {
            id,
            started_on,
            project_name: String::from(project_name),
        }
    }
}

fn get_running_slice(conn: &Connection) -> Result<Option<RunningTimeslice>, Box<dyn Error>> {
    match conn
        .query_row::<RunningTimeslice, _, _>(
            "SELECT t.timeslice_id, t.started_on, p.title FROM timeslice t JOIN project p WHERE t.stopped_on IS NULL",
            NO_PARAMS,
            |row| Ok(RunningTimeslice::new(
                 row.get(0)?,
                 &*row.get::<_, String>(1)?,
                 &*row.get::<_, String>(1)?))
        )
        .optional()? {
        Some(slice) => Ok(Some(slice)),
        _ => Ok(None),
    }
}

pub fn start_command(conn: &Connection) -> Result<(), Box<dyn Error>> {
    match get_running_slice(conn)? {
        Some(slice) => println!("found running slice: {:?}", slice),
        None => println!("no running slice found."),
    };
    Ok(())
}
