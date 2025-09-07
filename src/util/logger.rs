use log::{LevelFilter, Log, SetLoggerError};

/// Minimal logger implementation with verbosity/quietness arguments.
/// Checking the verbosity argument before every print was cumbersone so I decided to
/// automate this with `log`. The available crates implementing simple loggers prepended
/// the log level before printing message, and this couldn't be disabled. I found this undesirable
/// so I wrote this implementation.
pub struct Logger {
    filter: LevelFilter,
}
impl Logger {
    pub fn new(verbosity: u8, quietness: u8) -> Self {
        let net_verbosity = (verbosity as isize) - (quietness as isize);
        let filter = match net_verbosity {
            x if x <= -2 => LevelFilter::Off,
            -1 => LevelFilter::Error,
            0 => LevelFilter::Warn,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };
        Self { filter }
    }
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(LevelFilter::Trace);
        log::set_boxed_logger(Box::new(self))
    }
}
impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.filter >= metadata.level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Some(s) = record.args().as_str() {
                eprintln!("{}", s);
            } else {
                eprintln!("{}", record.args());
            }
        }
    }

    fn flush(&self) {}
}
