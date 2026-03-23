use log::{LevelFilter, Log, Metadata, Record, debug, error, info, max_level, trace, warn};

pub struct PrintingLogger;
impl Log for PrintingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}

#[derive(Debug)]
pub struct TanglLogger;
impl TanglLogger {
    pub fn new() -> Self {
        Self
    }
    pub fn trace<S: Into<String>>(&self, message: S) {
        self.log(message, LevelFilter::Trace)
    }
    pub fn debug<S: Into<String>>(&self, message: S) {
        self.log(message, LevelFilter::Debug)
    }
    pub fn info<S: Into<String>>(&self, message: S) {
        self.log(message, LevelFilter::Info)
    }
    pub fn warn<S: Into<String>>(&self, message: S) {
        self.log(message, LevelFilter::Warn)
    }
    pub fn error<S: Into<String>>(&self, message: S) {
        self.log(message, LevelFilter::Error)
    }

    fn log<S: Into<String>>(&self, message: S, level: LevelFilter) {
        let converted = message.into();
        if !converted.is_empty() {
            match level {
                LevelFilter::Error => error!("{}", converted),
                LevelFilter::Warn => warn!("{}", converted),
                LevelFilter::Info => info!("{}", converted),
                LevelFilter::Debug => debug!("{}", converted),
                LevelFilter::Trace => trace!("{}", converted),
                LevelFilter::Off => {}
            }
        }
    }
}
