// #[allow(unused_variables, unused_imports)]
use chrono::{Duration, Utc};
use clap::{App, AppSettings, Arg, SubCommand};
use rusqlite::{Connection, Result, NO_PARAMS};
use std::error::Error;
use std::path::{Path, PathBuf};
// use std::result::Result;
use xdg;

mod colors;
mod datetime;
mod db;
mod filter;
mod import;
mod log;
mod migration;
mod schema;
mod startstop;
mod summarize;
mod tinylogger;

fn get_default_db_filename() -> PathBuf {
    let xdirs = xdg::BaseDirectories::with_prefix("punch").unwrap();
    xdirs.place_data_file("punch.sqlite").unwrap()
}

fn get_db_filename<'a>(default_value: PathBuf, option_value: Option<&'a str>) -> PathBuf {
    match option_value {
        Some(v) => Path::new(v).to_path_buf(),
        None => default_value,
    }
}

fn get_connection(filename: PathBuf) -> Result<Connection> {
    let mut conn = Connection::open(filename)?;
    rusqlite::vtab::array::load_module(&conn)?;
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
        .subcommand(
            SubCommand::with_name("log")
                .about("log slices created during the last 7d")
                .arg(
                    Arg::with_name("all")
                        .help("log all recorded slices")
                        .short("a")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("start logging time")
                .arg(
                    Arg::with_name("project")
                        .required(true)
                        .help("project name"),
                )
                .arg(
                    Arg::with_name("tag")
                        .help("tags")
                        .multiple(true)
                        .takes_value(true)
                        .short("t"),
                ),
        )
        .subcommand(SubCommand::with_name("stop").about("stop currently running slice"))
        .subcommand(
            SubCommand::with_name("summarize")
                .about("Summarize work by project and time period. By default the output is grouped by day.")
                .arg(
                    Arg::with_name("all")
                        .help("Create only one group containing all work ever recorded")
                        .short("a")
                        .required(false),
                ),
        )
        .get_matches();

    let db_filename = get_db_filename(get_default_db_filename(), matches.value_of("dbfile"));

    tinylogger::init(matches.is_present("verbose"))?;

    if let Some(import_matches) = matches.subcommand_matches("import") {
        if let Some(import_file) = import_matches.value_of("file") {
            println!("importing from file: {}", import_file);
            let mut conn = get_connection(db_filename.clone())?;
            import::import_watson_frames(&mut conn, import_file)?;
        }
    }

    if let Some(log_matches) = matches.subcommand_matches("log") {
        let from = Some(if log_matches.is_present("all") {
            datetime::timestamp_1970()
        } else {
            Utc::now() - Duration::days(7)
        });

        let filter = filter::Filter { from, to: None };

        log::log_command(&mut get_connection(db_filename.clone())?, &filter)?;
    }

    if let Some(start_matches) = matches.subcommand_matches("start") {
        if let Some(project_name) = start_matches.value_of("project") {
            let tags = match start_matches.values_of("tag") {
                Some(tags) => tags.collect(),
                None => vec![],
            };
            startstop::start_command(
                &mut get_connection(db_filename.clone())?,
                project_name,
                &tags,
            )?;
        }
    }

    if let Some(_args) = matches.subcommand_matches("stop") {
        startstop::stop_command(&mut get_connection(db_filename.clone())?)?;
    }

    if let Some(summarize_matches) = matches.subcommand_matches("summarize") {
        let grouping_mode = if summarize_matches.is_present("all") {
            summarize::GroupingMode::All
        } else {
            summarize::GroupingMode::Day
        };
        summarize::summarize_command(&mut get_connection(db_filename.clone())?, grouping_mode)?;
    }

    Ok(())
}
