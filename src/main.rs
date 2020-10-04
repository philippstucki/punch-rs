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
mod summarize;
mod tinylogger;
mod colors;

const DEFAULT_DB_FILE: &'static str = "./punch.sqlite";

fn get_db_filename<'a>(default_value: &'a str, option_value: Option<&'a str>) -> &'a str {
    match option_value {
        Some(v) => v,
        None => default_value,
    }
}

fn get_connection(filename: &str) -> Result<Connection> {
    let mut conn = Connection::open(Path::new(filename))?;
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
        .arg(
            Arg::with_name("verbose")
                .global(true)
                .short("v")
                .long("verbose")
                .help("enables logging of debug messages"),
        )
        .arg(
            Arg::with_name("dbfile")
                .global(true)
                .short("f")
                .long("dbfile")
                .takes_value(true)
                .value_name("file")
                .help("database file to use. defaults to ./punch.sqlite"),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("import frames from watson")
                .arg(
                    Arg::with_name("file")
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
        .subcommand(
            SubCommand::with_name("summarize").about("summarize work by project and time period"),
        )
        .get_matches();

    let db_filename = get_db_filename(DEFAULT_DB_FILE, matches.value_of("dbfile"));

    tinylogger::init(matches.is_present("verbose"))?;

    if let Some(import_matches) = matches.subcommand_matches("import") {
        if let Some(import_file) = import_matches.value_of("file") {
            println!("importing from file: {}", import_file);
            let mut conn = get_connection(db_filename)?;
            import::import_watson_frames(&mut conn, import_file)?;
        }
    }

    if let Some(_args) = matches.subcommand_matches("log") {
        log::log_command(&mut get_connection(db_filename)?)?;
    }

    if let Some(start_matches) = matches.subcommand_matches("start") {
        if let Some(project_name) = start_matches.value_of("project") {
            startstop::start_command(&mut get_connection(db_filename)?, project_name)?;
        }
    }

    if let Some(_args) = matches.subcommand_matches("stop") {
        startstop::stop_command(&mut get_connection(db_filename)?)?;
    }

    if let Some(_args) = matches.subcommand_matches("summarize") {
        summarize::summarize_command(&mut get_connection(db_filename)?)?;
    }

    Ok(())
}
