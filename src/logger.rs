use std::sync::mpsc;

use eframe::WebLogger;

pub type Transmitted = (log::Level, String);

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
        let send_result = self
            .log_sender
            .send((record.level(), record.args().to_string()));

        // Inform of applocation logging failure.
        if let Err(_) = send_result {
            let warn_log = log::Record::builder()
                .level(log::Level::Warn)
                .args(format_args!("Unable to send previous log to application."))
                .build();
            self.web_logger.log(&warn_log);
        }
    }

    fn flush(&self) {
        self.web_logger.flush();
    }
}
