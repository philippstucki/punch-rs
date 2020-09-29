// #[allow(unused_variables, unused_imports)]
use clap::{App, AppSettings, Arg, SubCommand};
use rusqlite::{Connection, Result, NO_PARAMS};
use std::error::Error;
use std::path::Path;

mod datetime;
mod db;
mod import;
mod log;
mod migration;
mod schema;
mod startstop;

const DB_FILE: &'static str = "./punch.sqlite";

fn get_connection() -> Result<Connection> {
    let mut conn = Connection::open(Path::new(DB_FILE))?;
    conn.execute("PRAGMA foreign_keys = ON;", NO_PARAMS)?;
    schema::migrate(&mut conn)?;
    Ok(conn)
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Punch")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version("0.1")
        .author("Philipp Stucki <ps@stuckistucki.com>")
        .about("A cli based time logger")
        .subcommand(
            SubCommand::with_name("import")
                .about("import frames from watson")
                .arg(
                    Arg::with_name("FILE")
                        .required(true)
                        .index(1)
                        .help("input file to use for import"),
                ),
        )
        .subcommand(SubCommand::with_name("log").about("log recent work"))
        .subcommand(
            SubCommand::with_name("start")
                .about("start logging time")
                .arg(
                    Arg::with_name("project")
                        .required(true)
                        .index(1)
                        .help("project name"),
                ),
        )
        .subcommand(SubCommand::with_name("stop").about("stop currently running slice"))
        .get_matches();

    if let Some(import_matches) = matches.subcommand_matches("import") {
        if let Some(import_file) = import_matches.value_of("FILE") {
            println!("importing from file: {}", import_file);
            let mut conn = get_connection()?;
            import::import_watson_frames(&mut conn, import_file)?;
        }
    }

    if let Some(_args) = matches.subcommand_matches("log") {
        log::log_command(&mut get_connection()?)?;
    }

    if let Some(start_matches) = matches.subcommand_matches("start") {
        if let Some(project_name) = start_matches.value_of("project") {
            startstop::start_command(&mut get_connection()?, project_name)?;
        }
    }

    if let Some(_args) = matches.subcommand_matches("stop") {
        startstop::stop_command(&mut get_connection()?)?;
    }

    Ok(())
}
