use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct TinyLogger {
    log_level: Level,
}

impl log::Log for TinyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init(verbose: bool) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(TinyLogger {
        log_level: match verbose {
            true => Level::Debug,
            false => Level::Info,
        },
    }))
    .map(|()| log::set_max_level(LevelFilter::Debug))
}
