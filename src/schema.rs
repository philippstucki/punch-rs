use crate::migration;
use rusqlite::{Connection, Result, NO_PARAMS};

fn migration_1_initial_structure(conn: &Connection) -> Result<bool> {
    println!("Executing migration 1: initial structure");

    conn.execute(
        "CREATE TABLE project (
            project_id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            CONSTRAINT name_unique UNIQUE (name)
        )",
        NO_PARAMS,
    )?;
    conn.execute(
        "CREATE TABLE tag (
            tag_id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            project_id INTEGER NOT NULL,
            FOREIGN KEY (project_id) REFERENCES project (project_id),
            CONSTRAINT name_unique UNIQUE (name)
        )",
        NO_PARAMS,
    )?;
    conn.execute(
        "CREATE TABLE timeslice (
            id INTEGER PRIMARY KEY NOT NULL,
            started_on DATETIME NOT NULL,
            stopped_on DATETIME
        )",
        NO_PARAMS,
    )?;

    Ok(true)
}

pub fn migrate(conn: &mut Connection) -> Result<()> {
    migration::create_schema_migrations_table(&conn)?;
    migration::execute_migration(conn, 1, migration_1_initial_structure)?;
    Ok(())
}
