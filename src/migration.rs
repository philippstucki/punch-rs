use rusqlite::{Connection, OptionalExtension, Result, NO_PARAMS};

fn create_schema_migrations_table(db: &Connection) -> Result<usize> {
    println!("Creating schema migrations table...");
    db.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations(
        id INTEGER PRIMARY KEY NOT NULL,
        executed_on DATETIME
      );",
        NO_PARAMS,
    )
}

fn has_migration(db: &Connection, id: u64) -> Result<bool> {
    println!("Checking for migration #{}", id);
    let res: Option<bool> = db
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

fn migration_1_timeslice(db: &Connection) -> Result<()> {
    if !has_migration(db, 1)? {
        println!("Executing migration 1: timeslices");
    }
    Ok(())
}

pub fn migrate(db: &Connection) -> Result<()> {
    create_schema_migrations_table(&db)?;
    migration_1_timeslice(db)?;
    Ok(())
}

// format!()
