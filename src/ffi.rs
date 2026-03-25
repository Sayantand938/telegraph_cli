//! C-style FFI for tx_tracker
//!
//! This module provides C-compatible functions for FFI usage.
//! All functions accept JSON strings and return JSON strings.
//!
//! Memory management: All strings returned by FFI functions are owned by the library
//! and must be freed using the corresponding free functions.

use crate::{handle_json, Tracker};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use std::sync::{Mutex, OnceLock};

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static STRING_STORE: OnceLock<Mutex<Vec<CString>>> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

fn get_string_store() -> &'static Mutex<Vec<CString>> {
    STRING_STORE.get_or_init(|| Mutex::new(Vec::new()))
}

/// Store a string and return a pointer to it
/// The string will live for the lifetime of the process
fn store_string(s: String) -> *const c_char {
    let store = get_string_store();
    let mut vec = store.lock().unwrap();
    vec.push(CString::new(s).unwrap());
    vec.last().unwrap().as_ptr()
}

/// Opaque handle to a Tracker instance
pub struct TrackerHandle {
    tracker: Tracker,
}

/// Initialize the library and create a tracker instance.
///
/// # Arguments
/// * `db_path` - Path to database file, or null for default location
///
/// # Returns
/// Pointer to TrackerHandle, or null on error
#[unsafe(no_mangle)]
pub extern "C" fn tracker_init(db_path: *const c_char) -> *mut TrackerHandle {
    let db_path_opt = if db_path.is_null() {
        None
    } else {
        unsafe {
            CStr::from_ptr(db_path)
                .to_str()
                .ok()
                .map(|s| PathBuf::from(s.to_string()))
        }
    };

    let rt = get_runtime();
    match rt.block_on(async {
        let tracker = Tracker::new(db_path_opt).await?;
        Ok::<TrackerHandle, crate::AppError>(TrackerHandle { tracker })
    }) {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a tracker instance.
///
/// # Safety
/// `handle` must be a valid pointer returned by tracker_init
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_free(handle: *mut TrackerHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);
    }
}

/// Send a request to the tracker and get a response.
/// The returned string is owned by the library and must be freed with tracker_response_free.
///
/// # Arguments
/// * `handle` - Pointer to TrackerHandle
/// * `json_request` - JSON request string (null-terminated)
///
/// # Returns
/// JSON response string (caller must free with tracker_response_free),
/// or null on error
///
/// # Safety
/// `handle` and `json_request` must be valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_request(
    handle: *mut TrackerHandle,
    json_request: *const c_char,
) -> *const c_char {
    if handle.is_null() || json_request.is_null() {
        return ptr::null();
    }

    let request_str = match CStr::from_ptr(json_request).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    let tracker = &(*handle).tracker;
    let rt = get_runtime();

    let request: crate::Request = match serde_json::from_str(request_str) {
        Ok(r) => r,
        Err(_) => return ptr::null(),
    };

    let response = rt.block_on(tracker.handle(&request));

    match serde_json::to_string(&response) {
        Ok(json) => store_string(json),
        Err(_) => ptr::null(),
    }
}

/// Free a response string returned by tracker_request.
/// Note: Currently a no-op as strings are stored in the library's string store.
/// Provided for API compatibility.
///
/// # Safety
/// `response` must be a valid pointer returned by tracker_request or tracker_handle_json
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_response_free(_response: *const c_char) {
    // No-op: strings are stored in the library's string store for the lifetime of the process
    // This function is provided for API compatibility and may be implemented in the future
}

/// Convenience function: JSON in, JSON out (no handle management needed).
/// Creates and destroys a tracker internally.
/// The returned string is owned by the library and must be freed with tracker_response_free.
///
/// # Arguments
/// * `json_request` - JSON request string (null-terminated)
/// * `db_path` - Path to database file, or null for default location
///
/// # Returns
/// JSON response string (caller must free with tracker_response_free),
/// or null on error
///
/// # Safety
/// `json_request` and `db_path` must be valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_handle_json(
    json_request: *const c_char,
    db_path: *const c_char,
) -> *const c_char {
    if json_request.is_null() {
        return ptr::null();
    }

    let request_str = match CStr::from_ptr(json_request).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    let db_path_opt = if db_path.is_null() {
        None
    } else {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => Some(PathBuf::from(s.to_string())),
            Err(_) => return ptr::null(),
        }
    };

    let rt = get_runtime();
    let response_json = rt.block_on(handle_json(request_str, db_path_opt));

    store_string(response_json)
}

/// Get the library version string.
///
/// # Returns
/// Static string with version (no need to free)
#[unsafe(no_mangle)]
pub extern "C" fn tracker_version() -> *const c_char {
    c"0.1.0".as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_roundtrip() {
        let request = CString::new(r#"{"tool":"list_transactions","args":null}"#).unwrap();

        unsafe {
            let handle = tracker_init(ptr::null());
            assert!(!handle.is_null());

            let response_ptr = tracker_request(handle, request.as_ptr());
            assert!(!response_ptr.is_null());

            let response = CStr::from_ptr(response_ptr).to_str().unwrap();
            assert!(response.contains("\"success\":true"));

            tracker_response_free(response_ptr);
            tracker_free(handle);
        }
    }

    #[test]
    fn test_handle_json() {
        let request = CString::new(r#"{"tool":"list_activities","args":null}"#).unwrap();

        unsafe {
            let response_ptr = tracker_handle_json(request.as_ptr(), ptr::null());
            assert!(!response_ptr.is_null());

            let response = CStr::from_ptr(response_ptr).to_str().unwrap();
            assert!(response.contains("\"success\":true"));

            tracker_response_free(response_ptr);
        }
    }
}
