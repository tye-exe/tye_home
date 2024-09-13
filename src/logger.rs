use std::sync::mpsc;

use eframe::WebLogger;

pub type Transmitted = (log::Level, Option<&'static str>);

pub struct Logger {
    filter: log::LevelFilter,
    web_logger: WebLogger,

    log_sender: mpsc::Sender<Transmitted>,
}

impl Logger {
    /// Install a new `Logger`, piping all [`log`] events to the web console
    /// and to my application
    pub fn init(
        filter: log::LevelFilter,
    ) -> Result<mpsc::Receiver<Transmitted>, log::SetLoggerError> {
        let (tx, rx) = mpsc::channel();

        log::set_max_level(filter);
        log::set_boxed_logger(Box::new(Self::new(filter, tx)))?;

        Ok(rx)
    }

    /// Creates a new [`Logger`] with the given filter, but don't install it.
    pub fn new(filter: log::LevelFilter, log_sender: mpsc::Sender<Transmitted>) -> Self {
        Self {
            filter,
            web_logger: eframe::WebLogger::new(filter),
            log_sender,
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.filter
    }

    fn log(&self, record: &log::Record<'_>) {
        // Logs to js console.
        self.web_logger.log(record);

        // Logs to application.
        let _ = self
            .log_sender
            .send((record.level(), record.args().as_str()));
    }

    fn flush(&self) {
        self.web_logger.flush();
    }
}
