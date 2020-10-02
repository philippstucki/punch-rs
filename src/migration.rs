use rusqlite::{Connection, OptionalExtension, Result, NO_PARAMS};
use log::debug;

pub struct Migration<MFN>
where
    MFN: Fn(&Connection) -> Result<bool>,
{
    pub id: u64,
    pub migration_fn: MFN,
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

fn execute_migration<MF>(conn: &mut Connection, migration: Migration<MF>) -> Result<()>
where
    MF: Fn(&Connection) -> Result<bool>,
{
    if !has_migration(conn, migration.id)? {
        debug!("Checking for migration #{}", migration.id);
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

pub fn execute_migrations<MF>(conn: &mut Connection, migrations: Vec<Migration<MF>>) -> Result<()>
where
    MF: Fn(&Connection) -> Result<bool>,
{
    create_schema_migrations_table(&conn)?;

    migrations
        .into_iter()
        .map(|m| execute_migration(conn, m))
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
