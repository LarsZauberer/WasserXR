use std::fmt::Display;

use chrono::{DateTime, Local};

use crate::{scene::Scene, utils::ring::Ring};

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
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
pub struct LogEntry {
    level: LogLevel,
    logger: String,
    timestamp: DateTime<Local>,
    message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, logger: String, message: String) -> Self {
        let timestamp = Local::now();
        Self {
            level,
            logger,
            timestamp,
            message,
        }
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
pub struct LogManager {
    logs: Ring<LogEntry>,
    current_logger: String,
}

impl LogManager {
    pub fn new(logger: String) -> Self {
        Self {
            logs: Ring::new(300),
            current_logger: logger,
        }
    }

    pub fn set_logger(&mut self, name: String) {
        self.current_logger = name;
    }

    pub fn log(&mut self, level: LogLevel, message: String) {
        let entry = LogEntry::new(level, self.current_logger.clone(), message);
        self.logs.push(entry);
    }

    pub fn iter_logs(&self) -> std::collections::vec_deque::Iter<'_, LogEntry> {
        self.logs.iter()
    }
}

impl Scene {
    pub fn set_logger(&mut self, logger: String) {
        self.log_manager.set_logger(logger);
    }

    pub fn reset_logger(&mut self) {
        self.log_manager.set_logger("WasserXR".to_owned());
    }

    pub fn log(&mut self, level: LogLevel, message: String) {
        self.log_manager.log(level, message);
    }

    pub fn iter_logs(&self) -> std::collections::vec_deque::Iter<'_, LogEntry> {
        self.log_manager.iter_logs()
    }
}

#[macro_export]
macro_rules! debug {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::DEBUG, format!($($arg)+));
    }};
}

#[macro_export]
macro_rules! info {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::INFO, format!($($arg)+));
    }};
}

#[macro_export]
macro_rules! warn {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::WARN, format!($($arg)+));
    }};
}

#[macro_export]
macro_rules! error {
    ($scene:expr, $($arg:tt)+) => {{
        $scene.log($crate::scene::logging::LogLevel::ERROR, format!($($arg)+));
    }};
}
