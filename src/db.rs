use rusqlite::Connection;

#[derive(Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

pub fn get_project_by_name(conn: &Connection, name: String) -> Option<Project> {
    match conn.query_row(
        "SELECT id, name FROM project WHERE name = ?",
        &[name],
        |row| {
            Ok(Some(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            }))
        },
    ) {
        Err(_) => None,
        Ok(p) => p,
    }
}
