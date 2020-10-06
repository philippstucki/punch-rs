use chrono::{DateTime, Utc};
use rusqlite::{named_params, params, Connection, OptionalExtension, Result};

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
    pub stopped_on: Option<DateTime<Utc>>,
}
pub fn timeslice_create(conn: &Connection, timeslice: Timeslice) -> Result<i64> {
    let mut params: Vec<&dyn rusqlite::ToSql> =
        params![timeslice.project_id, timeslice.started_on].to_vec();

    if let Some(stopped_on) = timeslice.stopped_on.as_ref() {
        params.push(stopped_on);
    } else {
        params.push(&rusqlite::types::Null);
    }

    conn.execute(
        "INSERT INTO timeslice (project_id, started_on, stopped_on) VALUES (?1, ?2, ?3);",
        params,
    )?;
    Ok(conn.last_insert_rowid())
}

// tag
/////////////////////////////
pub fn tag_get_id_by_name_and_project_id(
    conn: &Connection,
    title: &str,
    project_id: i64,
) -> Result<Option<i64>> {
    Ok(conn
        .query_row_named(
            "SELECT tag_id FROM tag WHERE title = :title AND project_id = :project_id",
            named_params! {":title": title, ":project_id": project_id},
            |row| row.get(0),
        )
        .optional()?)
}

pub struct TagCreate {
    pub title: String,
    pub project_id: i64,
}
pub fn tag_create(conn: &Connection, tag: TagCreate) -> Result<i64> {
    conn.execute_named(
        "INSERT INTO tag (title, project_id) VALUES (:title, :project_id)",
        named_params! {":title":tag.title, ":project_id":tag.project_id},
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn tag_get_id_or_create(conn: &Connection, tag: TagCreate) -> Result<i64> {
    Ok(
        match tag_get_id_by_name_and_project_id(conn, &tag.title, tag.project_id)? {
            None => tag_create(conn, tag)?,
            Some(t) => t,
        },
    )
}

pub struct TimesliceTagCreate {
    pub tag_id: i64,
    pub timeslice_id: i64,
}
pub fn tag_assign_to_timeslice(
    conn: &Connection,
    timeslice_tag: TimesliceTagCreate,
) -> Result<i64> {
    conn.execute_named(
        "INSERT INTO timeslice_tag (tag_id, timeslice_id) VALUES (:tag_id, :timeslice_id)",
        named_params! {":tag_id": timeslice_tag.tag_id, ":timeslice_id": timeslice_tag.timeslice_id},
    )?;
    Ok(conn.last_insert_rowid())
}
