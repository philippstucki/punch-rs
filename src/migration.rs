use rusqlite::{Connection, OptionalExtension, Result, NO_PARAMS};

// mod migration;

pub fn create_schema_migrations_table(conn: &Connection) -> Result<usize> {
    println!("Creating schema migrations table...");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations(
        id INTEGER PRIMARY KEY NOT NULL,
        executed_on DATETIME
      );",
        NO_PARAMS,
    )
}

fn has_migration(conn: &Connection, id: u64) -> Result<bool> {
    println!("Checking for migration #{}", id);
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

pub fn execute_migration<MF>(conn: &mut Connection, id: u64, migration: MF) -> Result<()>
where
    MF: Fn(&Connection) -> Result<bool>,
{
    if !has_migration(conn, id)? {
        let tx = conn.transaction()?;
        migration(&tx)?;
        tx.execute(
            "INSERT INTO schema_migrations (id, executed_on) VALUES (?, date('now'))",
            &[id.to_string()],
        )?;
        tx.commit()?;
    }
    Ok(())
}
