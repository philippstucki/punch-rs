// #[allow(unused_variables, unused_imports)]
use rusqlite::Connection;
use std::path::Path;
mod migration;

const DB_FILE: &'static str = "./punch.sqlite";

fn get_connection() -> Connection {
    Connection::open(Path::new(DB_FILE))
        .unwrap_or_else(|error| panic!("Unable to open DB: {:?}", error))
}

fn main() {
    let db = get_connection();
    migration::migrate(&db);
}
