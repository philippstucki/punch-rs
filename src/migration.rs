use rusqlite::{Connection, Result, NO_PARAMS};

fn has_schema_migrations(db: &Connection) -> Result<u16> {
    let r = db.query_row(
        "SELECT * from pragma_table_info('schema_migrations')",
        NO_PARAMS,
        |row| row.get(0),
    );
    println!("{:?}", r);
    r
}

fn create_schema_migrations_table(db: &Connection) -> Result<usize> {
    println!("Creating schema migrations table...");
    db.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations(
        id INTEGER PRIMARY KEY NOT NULL,
        executed DATETIME
      );",
        NO_PARAMS,
    )
}

fn initialize_migration(db: &Connection) {
    if let Err(_) = has_schema_migrations(&db) {
        let _ = create_schema_migrations_table(&db);
    }
}

pub fn migrate(db: &Connection) {
    initialize_migration(&db)
}
