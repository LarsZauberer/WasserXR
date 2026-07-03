use std::{cell::RefCell, fmt::Display, fs::OpenOptions, io::Write};

use chrono::{DateTime, Local};

use crate::{scene::Scene, utils::ring::Ring};

/// Callback invoked for every log entry before the entry is stored.
pub type LogCallback = fn(&LogEntry);

#[derive(Debug, Clone, Copy)]
/// Log severity used by WasserXR's scene logger.
pub enum LogLevel {
    /// Detailed diagnostic message.
    DEBUG,
    /// Normal runtime information.
    INFO,
    /// Recoverable problem.
    WARN,
    /// Error condition.
    ERROR,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
        };
        write!(f, "{}", text)
    }
}

#[derive(Clone)]
/// One formatted scene log entry.
pub struct LogEntry {
    level: LogLevel,
    logger: String,
    timestamp: DateTime<Local>,
    message: String,
}

impl LogEntry {
    /// Creates a log entry with the current local timestamp.
    pub fn new(level: LogLevel, logger: String, message: String) -> Self {
        let timestamp = Local::now();
        Self {
            level,
            logger,
            timestamp,
            message,
        }
    }

    /// Returns the entry severity.
    pub fn get_level(&self) -> LogLevel {
        self.level
    }

    /// Returns the log message.
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// Returns the logger name that produced this entry.
    pub fn get_logger(&self) -> &str {
        &self.logger
    }

    /// Returns the timestamp captured when the entry was created.
    pub fn get_timestamp(&self) -> &DateTime<Local> {
        &self.timestamp
    }
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = self.timestamp.format("%H:%M:%S");
        write!(
            f,
            "[{}][{}][{}]: {}",
            time, self.level, self.logger, self.message
        )
    }
}

#[derive(Clone)]
/// In-memory scene log buffer with optional callbacks.
pub struct LogManager {
    logs: RefCell<Ring<LogEntry>>,
    current_logger: String,
    callbacks: Vec<LogCallback>,
}

impl LogManager {
    /// Creates a logger with an initial logger name.
    pub fn new(logger: String) -> Self {
        Self {
            logs: RefCell::new(Ring::new(300)),
            current_logger: logger,
            callbacks: Vec::new(),
        }
    }

    /// Sets the logger name attached to future entries.
    pub fn set_logger(&mut self, name: String) {
        self.current_logger = name;
    }

    /// Records a log entry and calls registered callbacks.
    pub fn log(&self, level: LogLevel, message: String) {
        let entry = LogEntry::new(level, self.current_logger.clone(), message);

        for i in &self.callbacks {
            i(&entry)
        }

        self.logs.borrow_mut().push(entry);
    }

    /// Returns a snapshot iterator over stored log entries from oldest to newest.
    pub fn iter_logs(&self) -> std::vec::IntoIter<LogEntry> {
        self.logs
            .borrow()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Registers a callback for future log entries.
    pub fn register_callback(&mut self, callback: LogCallback) {
        self.callbacks.push(callback);
    }
}

impl Scene {
    /// Sets the logger name used by `Scene::log`.
    pub fn set_logger(&mut self, logger: String) {
        self.log_manager.set_logger(logger);
    }

    /// Resets the logger name to `WasserXR`.
    pub fn reset_logger(&mut self) {
        self.log_manager.set_logger("WasserXR".to_owned());
    }

    /// Writes a log entry to the scene log.
    ///
    /// # Examples
    ///
    /// ```
    /// let scene = wasserxr::scene::Scene::new();
    ///
    /// scene.log(wasserxr::scene::logging::LogLevel::INFO, "ready".to_owned());
    /// assert_eq!(scene.iter_logs().count(), 1);
    /// ```
    pub fn log(&self, level: LogLevel, message: String) {
        self.log_manager.log(level, message);
    }

    /// Returns a snapshot iterator over scene log entries.
    pub fn iter_logs(&self) -> std::vec::IntoIter<LogEntry> {
        self.log_manager.iter_logs()
    }

    /// Registers a callback for future scene log entries.
    pub fn register_callback_logger(&mut self, callback: LogCallback) {
        self.log_manager.register_callback(callback);
    }
}

/// Prints one log entry to stdout.
pub fn print_logger(entry: &LogEntry) {
    println!("{}", entry);
}

/// Appends one log entry to `log.log` in the current working directory.
pub fn file_logger(entry: &LogEntry) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log.log")
        .expect("failed to open log file");

    writeln!(file, "{}", entry).expect("failed to write log entry");
}

#[macro_export]
/// Logs a formatted `DEBUG` message through a scene.
macro_rules! debug {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::DEBUG, format!($($arg)+));
    }};
}

#[macro_export]
/// Logs a formatted `INFO` message through a scene.
macro_rules! info {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::INFO, format!($($arg)+));
    }};
}

#[macro_export]
/// Logs a formatted `WARN` message through a scene.
macro_rules! warn {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::WARN, format!($($arg)+));
    }};
}

#[macro_export]
/// Logs a formatted `ERROR` message through a scene.
macro_rules! error {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::ERROR, format!($($arg)+));
    }};
}
