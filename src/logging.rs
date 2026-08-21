//! This module defines a standardized logging system for WasserXR.
//!
//! It's main responsibility is to provide a logging interface that is accessible across the ABI
//! boundary.

use std::fmt::Display;

use crate::async_runtime::AsyncRuntimeHandle;

/// The log level enum describes the differnet log message logging level: Debug, Info, Warning,
/// Error, Critical.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum LogLevel {
    /// Very verboxe and often too send messages that should give a very in depth information on
    /// what the application is doing. It should be used to basically provide a program trace.
    #[default]
    Debug = 0,

    /// Information to the user that is not a problem but the user should notice it.
    Info = 1,

    /// A problem has occured that you can easily recover and the function can still fulfill it's
    /// duty but perhaps shouldn't have happened. The
    /// user should therefore be notified.
    Warning = 2,

    /// A more serious problem has occured and a function cannot fulfill it's duty anymore. It is
    /// not serious enough to panic, but it should very likely be handled by the user.
    Error = 3,

    /// A problem with no return has happened. Something has gone terribly wrong and the application
    /// cannot recover from it. Hence, the application should panic upon this call.
    Critical = 4,
}

/// This is a callback that executes whenever the [WasserXRLogEntry] executes
/// [WasserXRLogEntry::send_log].
///
/// For example, it could be a function that writes the log entries to a file.
type LogHandler = extern "C" fn(&WasserXRLogEntry) -> ();

/// This is an abstraction of a logging system interface for WasserXR.
///
/// It basically describes an observer pattern, where [LogHandler] are subscribed to the logger.
/// Whenever a new [WasserXRLogEntry] is sent by the logger, all [LogHandler] by the
/// [Logger:send_log] function, they will be called with an [wasserxr::async_runtime::AsyncRuntimeHandle].
///
/// There are convenience functions that can be easily used by the C-bindings to log something with
/// a corresponding level. These functions internally use the [Logger:send_log] by spawning a fire
/// and forget async task via the [wasserxr::async_runtime::AsyncRuntimeHandle]
pub(crate) trait Logger {
    /// This method should run all handler with the log by spawning blocking async tasks.
    ///
    /// We use blocking async tasks, because the [LogHandler] may cross the C-boundary, which cannot use
    /// the rust internal async systems and libraries.
    ///
    /// This method should be called asynchronously using the fire and forget pattern since it also
    /// doesn't provide any output.
    async fn send_log(&self, entry: &WasserXRLogEntry);

    fn add_handler(&mut self, handler: LogHandler);
    fn remove_handler(&mut self, handler: LogHandler);

    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn critical(&self, msg: &str);
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct WasserXRLogEntry {
    pub level: LogLevel,
    pub message: String,
}

impl Display for WasserXRLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.level, self.message)
    }
}

pub struct WasserXRLogger<R: AsyncRuntimeHandle> {
    runtime_handle: R,
    handlers: Vec<LogHandler>,
}

impl<R: AsyncRuntimeHandle> WasserXRLogger<R> {
    pub fn new(handle: R) -> Self {
        Self {
            runtime_handle: handle,
            handlers: Vec::new(),
        }
    }
}

impl<R: AsyncRuntimeHandle> Logger for WasserXRLogger<R> {
    async fn send_log(&self, entry: &WasserXRLogEntry) {
        for handler in self.handlers.iter().copied() {
            let entry = entry.clone();
            let _ = self
                .runtime_handle
                .spawn_blocking(move || handler(&entry))
                .await;
        }
    }

    fn add_handler(&mut self, handler: LogHandler) {
        self.handlers.push(handler);
    }

    fn remove_handler(&mut self, handler: LogHandler) {
        self.handlers
            .retain(|registered| !std::ptr::fn_addr_eq(*registered, handler));
    }

    fn debug(&self, msg: &str) {
        let entry = WasserXRLogEntry {
            level: LogLevel::Debug,
            message: msg.to_owned(),
        };
        for handler in self.handlers.iter().copied() {
            let entry = entry.clone();
            let _ = self.runtime_handle.spawn_blocking(move || handler(&entry));
        }
    }

    fn info(&self, msg: &str) {
        let entry = WasserXRLogEntry {
            level: LogLevel::Info,
            message: msg.to_owned(),
        };
        for handler in self.handlers.iter().copied() {
            let entry = entry.clone();
            let _ = self.runtime_handle.spawn_blocking(move || handler(&entry));
        }
    }

    fn warn(&self, msg: &str) {
        let entry = WasserXRLogEntry {
            level: LogLevel::Warning,
            message: msg.to_owned(),
        };
        for handler in self.handlers.iter().copied() {
            let entry = entry.clone();
            let _ = self.runtime_handle.spawn_blocking(move || handler(&entry));
        }
    }

    fn error(&self, msg: &str) {
        let entry = WasserXRLogEntry {
            level: LogLevel::Error,
            message: msg.to_owned(),
        };
        for handler in self.handlers.iter().copied() {
            let entry = entry.clone();
            let _ = self.runtime_handle.spawn_blocking(move || handler(&entry));
        }
    }

    fn critical(&self, msg: &str) {
        let entry = WasserXRLogEntry {
            level: LogLevel::Critical,
            message: msg.to_owned(),
        };
        for handler in self.handlers.iter().copied() {
            let entry = entry.clone();
            let _ = self.runtime_handle.spawn_blocking(move || handler(&entry));
        }

        panic!("{msg}");
    }
}

#[cfg(test)]
mod simple_logging {
    use std::sync::RwLock;

    use rstest::{fixture, rstest};

    use crate::logging::LogLevel;
    use crate::logging::Logger;
    use crate::logging::WasserXRLogEntry;
    use crate::logging::WasserXRLogger;

    static ENTRIES: RwLock<Vec<WasserXRLogEntry>> = RwLock::new(Vec::new());

    extern "C" fn log_handle(entry: &WasserXRLogEntry) {
        ENTRIES.write().unwrap().push(entry.clone());
    }

    #[fixture]
    fn simple_logger() -> WasserXRLogger<tokio::runtime::Handle> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        WasserXRLogger::new(runtime.handle().clone())
    }

    #[rstest]
    #[case(LogLevel::Debug)]
    #[case(LogLevel::Info)]
    #[case(LogLevel::Warning)]
    #[case(LogLevel::Error)]
    fn test_log_handling(
        mut simple_logger: WasserXRLogger<tokio::runtime::Handle>,
        #[case] level: LogLevel,
    ) {
        ENTRIES.write().unwrap().clear();
        simple_logger.add_handler(log_handle);

        match level {
            LogLevel::Debug => simple_logger.debug("Hello World!"),
            LogLevel::Info => simple_logger.info("Hello World!"),
            LogLevel::Warning => simple_logger.warn("Hello World!"),
            LogLevel::Error => simple_logger.error("Hello World!"),
            LogLevel::Critical => panic!("This test cannot handle panic!"),
        }

        let entry = ENTRIES.read().unwrap()[0].clone();
        assert_eq!(entry.level, LogLevel::Debug);
        assert_eq!(entry.message, "Hello World!");
    }

    #[rstest]
    #[should_panic]
    fn test_critical_panic(simple_logger: WasserXRLogger<tokio::runtime::Handle>) {
        simple_logger.critical("Hello World!");
    }
}
