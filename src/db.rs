use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result};

// project
/////////////////////////////
#[derive(Debug)]
pub struct Project {
    pub id: i64,
    pub title: String,
}

pub fn project_get_by_name(conn: &Connection, name: &str) -> Result<Option<Project>> {
    let res = conn
        .query_row(
            "SELECT project_id, title FROM project WHERE title = ?",
            &[name],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    title: row.get(1)?,
                })
            },
        )
        .optional()?;

    Ok(res)
}

pub fn project_create(conn: &Connection, title: &str) -> Result<i64> {
    conn.execute("INSERT INTO project (title) VALUES (?1)", params![title])?;
    Ok(conn.last_insert_rowid())
}

// timeslice
/////////////////////////////
pub struct Timeslice {
    pub id: Option<i64>,
    pub project_id: i64,
    pub started_on: DateTime<Utc>,
    pub stopped_on: DateTime<Utc>,
}
pub fn timeslice_create(conn: &Connection, timeslice: Timeslice) -> Result<i64> {
    conn.execute(
        "INSERT INTO timeslice (project_id, started_on, stopped_on) VALUES (?1, ?2, ?3);",
        params![
            timeslice.project_id,
            timeslice.started_on,
            timeslice.stopped_on
        ],
    )?;
    Ok(conn.last_insert_rowid())
}
