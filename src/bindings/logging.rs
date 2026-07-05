use std::{
    ffi::{c_char, c_longlong},
    ptr,
};

use crate::{
    bindings::{WXRSceneError, clear_error, set_error, str_from_ptr, string_to_ptr},
    scene::{
        Scene,
        logging::{LogEntry, LogLevel},
    },
};

use super::scene::WXRScene;

/// Opaque C handle for a scene log entry.
pub struct WXRLogEntry {
    _private: [u8; 0],
}

#[repr(C)]
/// Owned array of log entry handles.
pub struct WXRLogEntryArray {
    /// Pointer to the first log entry handle.
    pub ptr: *mut *mut WXRLogEntry,
    /// Number of log entries.
    pub len: usize,
}

fn scene_ref<'a>(scene: *const WXRScene) -> Result<&'a Scene, WXRSceneError> {
    if scene.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    Ok(unsafe { &*(scene as *const Scene) })
}

fn scene_mut<'a>(scene: *mut WXRScene) -> Result<&'a mut Scene, WXRSceneError> {
    if scene.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    Ok(unsafe { &mut *(scene as *mut Scene) })
}

fn entry_ref<'a>(entry: *const WXRLogEntry) -> Result<&'a LogEntry, WXRSceneError> {
    if entry.is_null() {
        return Err(WXRSceneError::NullPointer);
    }

    Ok(unsafe { &*(entry as *const LogEntry) })
}

/// Writes a log entry to a scene.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_log(
    scene: *const WXRScene,
    level: LogLevel,
    message: *const c_char,
) -> i32 {
    match (scene_ref(scene), unsafe { str_from_ptr(message) }) {
        (Ok(scene), Ok(message)) => {
            scene.log(level, message.to_owned());
            clear_error();
            0
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Sets the logger name used by future scene log entries.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_set_logger(scene: *mut WXRScene, logger: *const c_char) -> i32 {
    match (scene_mut(scene), unsafe { str_from_ptr(logger) }) {
        (Ok(scene), Ok(logger)) => {
            scene.set_logger(logger.to_owned());
            clear_error();
            0
        }
        (Err(error), _) | (_, Err(error)) => {
            set_error(error);
            -1
        }
    }
}

/// Returns a snapshot of stored scene log entries.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_iter_logs(scene: *const WXRScene) -> WXRLogEntryArray {
    match scene_ref(scene) {
        Ok(scene) => {
            let mut entries: Vec<*mut WXRLogEntry> = scene
                .iter_logs()
                .map(|entry| Box::into_raw(Box::new(entry)).cast())
                .collect();
            let array = WXRLogEntryArray {
                ptr: entries.as_mut_ptr(),
                len: entries.len(),
            };
            std::mem::forget(entries);
            clear_error();
            array
        }
        Err(error) => {
            set_error(error);
            WXRLogEntryArray {
                ptr: ptr::null_mut(),
                len: 0,
            }
        }
    }
}

/// Frees log entries returned by `wxr_iter_logs`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wxr_free_log_entries(array: WXRLogEntryArray) {
    if array.ptr.is_null() {
        return;
    }

    let entries = unsafe { Vec::from_raw_parts(array.ptr, array.len, array.len) };
    for entry in entries {
        if !entry.is_null() {
            unsafe {
                drop(Box::from_raw(entry as *mut LogEntry));
            }
        }
    }
}

/// Returns a log entry severity.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_log_entry_get_level(entry: *const WXRLogEntry) -> LogLevel {
    match entry_ref(entry) {
        Ok(entry) => {
            clear_error();
            entry.get_level()
        }
        Err(error) => {
            set_error(error);
            LogLevel::ERROR
        }
    }
}

/// Returns a log entry message as an owned C string.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_log_entry_get_message(entry: *const WXRLogEntry) -> *mut c_char {
    match entry_ref(entry) {
        Ok(entry) => {
            clear_error();
            string_to_ptr(entry.get_message().to_owned())
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Returns a log entry logger name as an owned C string.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_log_entry_get_logger(entry: *const WXRLogEntry) -> *mut c_char {
    match entry_ref(entry) {
        Ok(entry) => {
            clear_error();
            string_to_ptr(entry.get_logger().to_owned())
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

/// Returns a log entry Unix timestamp in seconds.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_log_entry_get_timestamp(entry: *const WXRLogEntry) -> c_longlong {
    match entry_ref(entry) {
        Ok(entry) => {
            clear_error();
            entry.get_timestamp().timestamp() as c_longlong
        }
        Err(error) => {
            set_error(error);
            0
        }
    }
}

/// Formats a log entry using WasserXR's display formatter.
#[unsafe(no_mangle)]
pub extern "C" fn wxr_render_log_entry(entry: *const WXRLogEntry) -> *mut c_char {
    match entry_ref(entry) {
        Ok(entry) => {
            clear_error();
            string_to_ptr(entry.to_string())
        }
        Err(error) => {
            set_error(error);
            ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::scene::{wxr_create_scene, wxr_destroy_scene};
    use std::ffi::{CStr, CString};

    #[test]
    fn log_entry_getters_and_render_work() {
        let scene = wxr_create_scene();
        let logger = CString::new("test").unwrap();
        let message = CString::new("ready").unwrap();

        assert_eq!(unsafe { wxr_set_logger(scene, logger.as_ptr()) }, 0);
        assert_eq!(
            unsafe { wxr_log(scene, LogLevel::INFO, message.as_ptr()) },
            0
        );

        let logs = wxr_iter_logs(scene);
        assert_eq!(logs.len, 1);
        let entry = unsafe { *logs.ptr };

        assert_eq!(wxr_log_entry_get_level(entry), LogLevel::INFO);
        let rendered = wxr_render_log_entry(entry);
        let rendered = unsafe { CStr::from_ptr(rendered) }
            .to_str()
            .unwrap()
            .to_owned();
        assert!(rendered.contains("[INFO][test]: ready"));

        unsafe {
            wxr_free_log_entries(logs);
            wxr_destroy_scene(scene);
        }
    }
}
