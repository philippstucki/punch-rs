// #[allow(unused_variables, unused_imports)]
use rusqlite::{Connection, Result, NO_PARAMS};
use std::error::Error;
use std::path::Path;

mod db;
mod import;
mod migration;
mod schema;

const DB_FILE: &'static str = "./punch.sqlite";

fn get_connection() -> Result<Connection> {
    let conn = Connection::open(Path::new(DB_FILE))?;
    conn.execute("PRAGMA foreign_keys = ON;", NO_PARAMS)?;
    Ok(conn)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = get_connection()?;
    schema::migrate(&mut conn)?;

    import::import_watson_frames(&mut conn)?;

    Ok(())
}
