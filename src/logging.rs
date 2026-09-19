//! Scene-owned asynchronous logging.

use std::{
    ffi::c_void,
    ptr, slice,
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use crate::utils::ring::Ring;

/// The severity of a log entry.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum LogLevel {
    /// Detailed diagnostic information.
    #[default]
    Debug = 0,
    /// General information about scene activity.
    Info = 1,
    /// A recoverable problem.
    Warning = 2,
    /// A failure that prevents an operation from completing.
    Error = 3,
}

/// An owned log entry with a C-compatible pointer/length message
/// representation.
///
/// The message pointer and length are owned by the entry and remain valid while
/// the entry is alive. Log handlers receive a borrowed pointer that is valid
/// only for the duration of their callback.
#[repr(C)]
#[derive(Debug)]
pub struct LogEntry {
    /// The entry severity.
    level: LogLevel,
    /// Pointer to the UTF-8 message bytes. WasserXR owns this allocation;
    /// callers must treat the pointer and its allocation as read-only.
    message: *const u8,
    /// Number of bytes at [`Self::message`]. This value must not be changed.
    message_len: usize,
}

impl LogEntry {
    fn new(level: LogLevel, message: String) -> Self {
        let bytes = message.into_bytes().into_boxed_slice();
        let message_len = bytes.len();
        let message = Box::into_raw(bytes) as *mut u8;
        Self {
            level,
            message,
            message_len,
        }
    }

    /// Returns the complete message as an owned Rust string.
    #[allow(non_snake_case)]
    pub fn getMessage(&self) -> String {
        // SAFETY: `message_bytes` is created from the valid UTF-8 `String`
        // allocation owned by this entry.
        unsafe { String::from_utf8_unchecked(self.message_bytes().to_vec()) }
    }

    /// Returns the entry severity.
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns the UTF-8 message bytes borrowed from this entry.
    pub fn message_bytes(&self) -> &[u8] {
        // SAFETY: LogEntry creates and owns a valid UTF-8 allocation for this
        // pointer/length pair, and the allocation remains alive for &self.
        unsafe { slice::from_raw_parts(self.message, self.message_len) }
    }
}

impl Clone for LogEntry {
    fn clone(&self) -> Self {
        Self::new(self.level, self.getMessage())
    }
}

impl Drop for LogEntry {
    fn drop(&mut self) {
        // SAFETY: `message` came from Box::into_raw for a boxed byte slice in
        // `new`, and this entry is the unique owner of that allocation.
        unsafe {
            drop(Box::from_raw(ptr::slice_from_raw_parts_mut(
                self.message as *mut u8,
                self.message_len,
            )));
        }
    }
}

// LogEntry owns the allocation behind its raw pointer and never shares it.
unsafe impl Send for LogEntry {}
unsafe impl Sync for LogEntry {}

/// A callback invoked on the scene's logger thread for each processed entry.
pub type LogCallback = unsafe extern "C" fn(data: *mut c_void, entry: *const LogEntry);

/// A log callback and its caller-owned opaque state.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LogHandler {
    /// The callback to invoke.
    pub callback: LogCallback,
    /// Opaque state passed to [`Self::callback`].
    pub data: *mut c_void,
}

#[derive(Debug)]
pub(crate) struct LogManager {
    sender: Option<Sender<LogEntry>>,
    handlers: Arc<RwLock<Vec<RegisteredHandler>>>,
    logs: Arc<RwLock<Ring<LogEntry>>>,
    logger: Option<JoinHandle<()>>,
}

// The join handle is only accessed through an exclusive reference while the
// manager is being dropped.
impl std::panic::RefUnwindSafe for LogManager {}

impl Default for LogManager {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handlers = Arc::new(RwLock::new(Vec::new()));
        let logs = Arc::new(RwLock::new(Ring::new(1024)));
        let logger = {
            let handlers = Arc::clone(&handlers);
            let logs = Arc::clone(&logs);
            thread::spawn(move || run_logger(receiver, handlers, logs))
        };

        Self {
            sender: Some(sender),
            handlers,
            logs,
            logger: Some(logger),
        }
    }
}

impl LogManager {
    pub(crate) fn log(&self, level: LogLevel, message: String) {
        self.sender
            .as_ref()
            .expect("logger sender missing")
            .send(LogEntry::new(level, message))
            .expect("logger thread stopped");
    }

    pub(crate) fn add_handler(&self, handler: LogHandler) {
        self.handlers
            .write()
            .expect("logger handler lock poisoned")
            .push(RegisteredHandler {
                callback: handler.callback,
                data: handler.data as usize,
            });
    }

    pub(crate) fn get_logs(&self) -> Ring<LogEntry> {
        self.logs.read().expect("logger log lock poisoned").clone()
    }
}

impl Drop for LogManager {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(logger) = self.logger.take() {
            let _ = logger.join();
        }
    }
}

fn run_logger(
    receiver: Receiver<LogEntry>,
    handlers: Arc<RwLock<Vec<RegisteredHandler>>>,
    logs: Arc<RwLock<Ring<LogEntry>>>,
) {
    while let Ok(entry) = receiver.recv() {
        let handlers = handlers
            .read()
            .expect("logger handler lock poisoned")
            .clone();
        for handler in handlers {
            // SAFETY: Registration is unsafe because the caller owns the
            // callback state and promises it remains valid until the scene is
            // dropped. The entry is borrowed only for this callback and is not
            // retained by the manager.
            unsafe {
                (handler.callback)(handler.data as *mut c_void, &entry);
            }
        }
        logs.write().expect("logger log lock poisoned").push(entry);
    }
}

#[derive(Debug, Clone)]
struct RegisteredHandler {
    callback: LogCallback,
    data: usize,
}
