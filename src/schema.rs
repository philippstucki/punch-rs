use crate::migration;
use rusqlite::{Connection, Result};

fn migration_1_initial_structure(conn: &Connection) -> Result<bool> {
    println!("Executing migration 1: initial structure");

    conn.execute_batch(
        "
        CREATE TABLE project (
            project_id INTEGER PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            CONSTRAINT name_unique UNIQUE (title)
        );
        CREATE INDEX project_title ON project (title);

        CREATE TABLE tag (
            tag_id INTEGER PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            project_id INTEGER NOT NULL,
            FOREIGN KEY (project_id) REFERENCES project (project_id)
        );
        CREATE UNIQUE INDEX tag_project_unique ON tag (title, project_id);

        CREATE TABLE timeslice (
            id INTEGER PRIMARY KEY NOT NULL,
            project_id INTEGER NOT NULL,
            started_on DATETIME NOT NULL,
            stopped_on DATETIME
        );
        ",
    )?;

    Ok(true)
}

pub fn migrate(conn: &mut Connection) -> Result<()> {
    let migrations = vec![migration::Migration {
        id: 1,
        migration_fn: migration_1_initial_structure,
    }];
    migration::execute_migrations(conn, migrations)?;
    Ok(())
}
