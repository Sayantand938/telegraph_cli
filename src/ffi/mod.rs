//! C-style FFI for tx_tracker

use crate::api::handle_json;
use crate::tracker::Tracker;
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

fn store_string(s: String) -> *const c_char {
    let store = get_string_store();
    let mut vec = store.lock().unwrap();
    vec.push(CString::new(s).unwrap());
    vec.last().unwrap().as_ptr()
}

pub struct TrackerHandle {
    tracker: Tracker,
}

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_free(handle: *mut TrackerHandle) {
    if !handle.is_null() {
        let _ = unsafe { Box::from_raw(handle) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_request(
    handle: *mut TrackerHandle,
    json_request: *const c_char,
) -> *const c_char {
    if handle.is_null() || json_request.is_null() {
        return ptr::null();
    }

    let request_str = match unsafe { CStr::from_ptr(json_request) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    let tracker = unsafe { &(*handle).tracker };
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_response_free(_response: *const c_char) {
    // No-op: strings stored in library's string store
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tracker_handle_json(
    json_request: *const c_char,
    db_path: *const c_char,
) -> *const c_char {
    if json_request.is_null() {
        return ptr::null();
    }

    let request_str = match unsafe { CStr::from_ptr(json_request) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    let db_path_opt = if db_path.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(db_path) }.to_str() {
            Ok(s) => Some(PathBuf::from(s.to_string())),
            Err(_) => return ptr::null(),
        }
    };

    let rt = get_runtime();
    let response_json = rt.block_on(handle_json(request_str, db_path_opt));

    store_string(response_json)
}

#[unsafe(no_mangle)]
pub extern "C" fn tracker_version() -> *const c_char {
    c"0.1.0".as_ptr()
}
