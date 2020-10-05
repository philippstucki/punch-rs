use log::debug;
use rusqlite::{Connection, OptionalExtension, Result, NO_PARAMS};

type MigrationFunction = fn(&Connection) -> Result<bool>;

pub struct Migration {
    pub id: u64,
    pub migration_fn: MigrationFunction,
}

pub fn create_schema_migrations_table(conn: &Connection) -> Result<usize> {
    debug!("Creating schema migrations table...");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations(
        id INTEGER PRIMARY KEY NOT NULL,
        executed_on DATETIME
      );",
        NO_PARAMS,
    )
}

fn has_migration(conn: &Connection, id: u64) -> Result<bool> {
    let res: Option<bool> = conn
        .query_row(
            "SELECT id, executed_on FROM schema_migrations WHERE id = ?",
            &[id.to_string()],
            |row| row.get(0),
        )
        .optional()?;
    match res {
        Some(v) => Ok(v),
        None => Ok(false),
    }
}

fn execute_migration(conn: &mut Connection, migration: Migration) -> Result<()> {
    if !has_migration(conn, migration.id)? {
        debug!("applying migration #{}", migration.id);
        let tx = conn.transaction()?;
        (migration.migration_fn)(&tx)?;
        tx.execute(
            "INSERT INTO schema_migrations (id, executed_on) VALUES (?, datetime('now'))",
            &[migration.id.to_string()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

pub fn execute_migrations(conn: &mut Connection, migrations: Vec<Migration>) -> Result<()> {
    create_schema_migrations_table(&conn)?;

    migrations
        .into_iter()
        .map(|m| execute_migration(conn, m))
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
