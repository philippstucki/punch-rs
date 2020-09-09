use rusqlite::{params, Connection, OptionalExtension, Result};

#[derive(Debug)]
pub struct Project {
    pub id: i64,
    pub title: String,
}

pub fn get_project_by_name(conn: &Connection, name: &str) -> Result<Option<Project>> {
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

pub fn create_project(conn: &Connection, title: &str) -> Result<i64> {
    conn.execute("INSERT INTO project (title) VALUES (?1)", params![title])?;
    Ok(conn.last_insert_rowid())
}
