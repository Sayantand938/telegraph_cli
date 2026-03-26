//! C-style FFI for logbook - Thread-Safe Implementation
//!
//! This module provides a thread-safe C FFI interface.
//! - Uses a multi-threaded Tokio runtime for async operations
//! - LogbookHandle can be safely shared across threads
//! - Response strings are properly tracked and can be freed individually

use crate::api::handle_json;
use crate::tracker::Tracker;
use crate::command::execute_command_with_db;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};

/// Global multi-threaded Tokio runtime
static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Thread-safe string store that tracks allocated strings
/// Uses a HashMap with incrementing IDs to allow individual string deallocation
static STRING_STORE: OnceLock<Arc<Mutex<StringStore>>> = OnceLock::new();

/// Manages allocated C strings with unique IDs for safe deallocation
struct StringStore {
    strings: HashMap<usize, CString>,
    next_id: usize,
}

impl StringStore {
    fn new() -> Self {
        Self {
            strings: HashMap::new(),
            next_id: 0,
        }
    }

    /// Store a string and return its ID encoded as a pointer
    fn store(&mut self, s: String) -> *const c_char {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        
        let cstring = CString::new(s).unwrap();
        let ptr = cstring.as_ptr();
        self.strings.insert(id, cstring);
        
        // Return the ID encoded as a pointer (not the actual string pointer)
        // The caller will use this ID to free the string
        ptr
    }

    /// Free a string by its ID
    fn free(&mut self, ptr: *const c_char) {
        // Find the ID by pointer and remove the string
        let id_to_remove = self.strings
            .iter()
            .find(|(_, cstr)| cstr.as_ptr() == ptr)
            .map(|(id, _)| *id);
        
        if let Some(id) = id_to_remove {
            self.strings.remove(&id);
        }
    }
}

fn get_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .thread_name("logbook-ffi")
            .build()
            .unwrap()
    })
}

fn get_string_store() -> Arc<Mutex<StringStore>> {
    STRING_STORE
        .get_or_init(|| Arc::new(Mutex::new(StringStore::new())))
        .clone()
}

/// Thread-safe logbook handle
/// Wrapped in Arc<Mutex<>> for safe concurrent access
pub struct LogbookHandle {
    tracker: Arc<Mutex<Tracker>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn logbook_init(db_path: *const c_char) -> *mut LogbookHandle {
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
        Ok::<LogbookHandle, crate::AppError>(LogbookHandle { 
            tracker: Arc::new(Mutex::new(tracker)) 
        })
    }) {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn logbook_free(handle: *mut LogbookHandle) {
    if !handle.is_null() {
        let _ = unsafe { Box::from_raw(handle) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn logbook_request(
    handle: *mut LogbookHandle,
    json_request: *const c_char,
) -> *const c_char {
    if handle.is_null() || json_request.is_null() {
        return ptr::null();
    }

    let request_str = match unsafe { CStr::from_ptr(json_request) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    let handle_ref = unsafe { &*handle };
    let tracker = handle_ref.tracker.clone();
    let rt = get_runtime();

    // Clone the request before entering the async block
    let request_str = request_str.to_string();
    
    let response = rt.block_on(async move {
        let tracker_guard = tracker.lock().unwrap();
        let request: crate::Request = match serde_json::from_str(&request_str) {
            Ok(r) => r,
            Err(_) => return None,
        };
        Some(tracker_guard.handle(&request).await)
    });

    match response {
        Some(resp) => {
            match serde_json::to_string(&resp) {
                Ok(json) => {
                    let store = get_string_store();
                    let mut store_guard = store.lock().unwrap();
                    store_guard.store(json)
                }
                Err(_) => ptr::null(),
            }
        }
        None => ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn logbook_response_free(response: *const c_char) {
    if !response.is_null() {
        let store = get_string_store();
        let mut store_guard = store.lock().unwrap();
        store_guard.free(response);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn logbook_handle_json(
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
    let request_str = request_str.to_string();
    let response_json = rt.block_on(async move {
        handle_json(&request_str, db_path_opt).await
    });

    let store = get_string_store();
    let mut store_guard = store.lock().unwrap();
    store_guard.store(response_json)
}

#[unsafe(no_mangle)]
pub extern "C" fn logbook_version() -> *const c_char {
    c"0.1.0".as_ptr()
}

/// Execute a simple command string (Flutter-friendly interface).
///
/// This is a convenience function that accepts a simple command string
/// instead of JSON. Perfect for Flutter/mobile apps.
///
/// Format: "<domain> <action> [--arg value...]"
///
/// Examples:
///   "transaction create --amount 50.0 --kind shopping --desc Groceries"
///   "todo list --status pending"
///   "journal search --query meeting"
///   "category list"
///
/// @param command Command string (null-terminated).
/// @param db_path Path to database file, or NULL for default location.
/// @return JSON response string (caller must free with logbook_response_free),
///         or NULL on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn logbook_command(
    command: *const c_char,
    db_path: *const c_char,
) -> *const c_char {
    if command.is_null() {
        return ptr::null();
    }

    let command_str = match unsafe { CStr::from_ptr(command) }.to_str() {
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
    let command_str = command_str.to_string();
    let response_json = rt.block_on(async move {
        execute_command_with_db(&command_str, db_path_opt.as_deref().map(|p| p.to_str()).flatten()).await
    });

    let store = get_string_store();
    let mut store_guard = store.lock().unwrap();
    store_guard.store(response_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use serde_json::Value;

    fn c_string(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_logbook_version_thread_safe() {
        let mut handles = vec![];
        
        for _ in 0..10 {
            let handle = thread::spawn(|| {
                let version = logbook_version();
                unsafe { CStr::from_ptr(version) }.to_str().unwrap().to_string()
            });
            handles.push(handle);
        }
        
        for handle in handles {
            assert_eq!(handle.join().unwrap(), "0.1.0");
        }
    }

    #[test]
    fn test_logbook_init_thread_safe() {
        let mut handles = vec![];
        
        for i in 0..5 {
            let handle = thread::spawn(move || {
                let db_path = c_string(&format!("test_thread_{}.db", i));
                let ptr = logbook_init(db_path.as_ptr());
                assert!(!ptr.is_null());
                
                // Clean up
                unsafe { logbook_free(ptr) };
                let _ = std::fs::remove_file(format!("test_thread_{}.db", i));
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_logbook_request_concurrent() {
        // Initialize separate trackers for each thread
        // This tests that the FFI runtime and string store are thread-safe
        let mut handles = vec![];
        
        for i in 0..5 {
            let thread_handle = thread::spawn(move || {
                // Each thread creates its own tracker
                let db_path = c_string(&format!("test_concurrent_{}.db", i));
                let handle = logbook_init(db_path.as_ptr());
                assert!(!handle.is_null());

                let request = c_string(&format!(
                    r#"{{"tool":"create_transaction","args":{{"amount":{},"kind":"concurrent_test","description":"Thread {}"}}}}"#,
                    10.0 + i as f64,
                    i
                ));

                let response = unsafe { logbook_request(handle, request.as_ptr()) };
                assert!(!response.is_null());

                let response_str = unsafe { CStr::from_ptr(response) }.to_str().unwrap();
                assert!(response_str.contains("\"success\":true"));

                // Free the response
                unsafe { logbook_response_free(response) };
                
                // Clean up tracker
                unsafe { logbook_free(handle) };
                let _ = std::fs::remove_file(format!("test_concurrent_{}.db", i));
            });
            handles.push(thread_handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_logbook_handle_json_concurrent() {
        let mut handles = vec![];
        
        for i in 0..5 {
            let thread_handle = thread::spawn(move || {
                let request = c_string(&format!(
                    r#"{{"tool":"create_transaction","args":{{"amount":{},"kind":"json_test","description":"JSON Thread {}"}}}}"#,
                    20.0 + i as f64,
                    i
                ));
                
                let response = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
                assert!(!response.is_null());
                
                let response_str = unsafe { CStr::from_ptr(response) }.to_str().unwrap();
                assert!(response_str.contains("\"success\":true"));
                
                // Free the response
                unsafe { logbook_response_free(response) };
            });
            handles.push(thread_handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_string_store_cleanup() {
        let store = get_string_store();
        let initial_count = store.lock().unwrap().strings.len();

        // Store some strings
        let ptr1 = {
            let mut guard = store.lock().unwrap();
            guard.store("test1".to_string())
        };
        let ptr2 = {
            let mut guard = store.lock().unwrap();
            guard.store("test2".to_string())
        };

        {
            let guard = store.lock().unwrap();
            assert_eq!(guard.strings.len(), initial_count + 2);
        }

        // Free one string
        {
            let mut guard = store.lock().unwrap();
            guard.free(ptr1);
        }

        {
            let guard = store.lock().unwrap();
            assert_eq!(guard.strings.len(), initial_count + 1);
        }

        // Free the other
        {
            let mut guard = store.lock().unwrap();
            guard.free(ptr2);
        }

        {
            let guard = store.lock().unwrap();
            assert_eq!(guard.strings.len(), initial_count);
        }
    }

    #[test]
    fn test_logbook_command_list_categories() {
        let command = c_string("category list");
        let response_ptr = unsafe { logbook_command(command.as_ptr(), ptr::null()) };
        assert!(!response_ptr.is_null());

        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();

        assert_eq!(json["success"], true);
        assert!(json["data"].is_array());

        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_create_transaction() {
        let command = c_string("transaction create --amount 99.99 --kind shopping --description FFI command test");
        let response_ptr = unsafe { logbook_command(command.as_ptr(), ptr::null()) };
        assert!(!response_ptr.is_null());

        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();

        assert_eq!(json["success"], true);
        assert!(json["data"]["id"].is_number());
        assert!(json["message"].as_str().unwrap().contains("created"));

        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_list_transactions() {
        let command = c_string("transaction list");
        let response_ptr = unsafe { logbook_command(command.as_ptr(), ptr::null()) };
        assert!(!response_ptr.is_null());

        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();

        assert_eq!(json["success"], true);
        assert!(json["data"].is_array());

        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_todo_list() {
        // List todos
        let list_cmd = c_string("todo list");
        let response_ptr = unsafe { logbook_command(list_cmd.as_ptr(), ptr::null()) };
        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["data"].is_array());
        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_journal_crud() {
        // Create a journal entry
        let create_cmd = c_string("journal create --content 'FFI command interface test entry'");
        let response_ptr = unsafe { logbook_command(create_cmd.as_ptr(), ptr::null()) };
        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        assert_eq!(json["success"], true);
        unsafe { logbook_response_free(response_ptr) };

        // List (not search, since FTS might need special setup)
        let list_cmd = c_string("journal list");
        let response_ptr = unsafe { logbook_command(list_cmd.as_ptr(), ptr::null()) };
        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["data"].is_array());
        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_invalid() {
        let command = c_string("invalid command here");
        let response_ptr = unsafe { logbook_command(command.as_ptr(), ptr::null()) };
        assert!(!response_ptr.is_null());

        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();

        assert_eq!(json["success"], false);
        assert!(json["error"].is_string());

        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_with_custom_db() {
        let db_path = c_string("test_command_custom.db");
        let command = c_string("category list");
        let response_ptr = unsafe { logbook_command(command.as_ptr(), db_path.as_ptr()) };
        assert!(!response_ptr.is_null());

        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        assert_eq!(json["success"], true);

        unsafe { logbook_response_free(response_ptr) };
        
        // Cleanup
        let _ = std::fs::remove_file("test_command_custom.db");
    }

    #[test]
    fn test_logbook_command_search_transactions() {
        // Create a transaction first
        let create_cmd = c_string("transaction create --amount 75.0 --kind shopping --description Search test");
        let response_ptr = unsafe { logbook_command(create_cmd.as_ptr(), ptr::null()) };
        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        assert_eq!(json["success"], true);
        unsafe { logbook_response_free(response_ptr) };

        // Search for it
        let search_cmd = c_string("search transaction --kind shopping --limit 10");
        let response_ptr = unsafe { logbook_command(search_cmd.as_ptr(), ptr::null()) };
        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        
        assert_eq!(json["success"], true);
        assert!(json["data"].is_array());

        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_search_todos() {
        // Search pending todos
        let search_cmd = c_string("search todo --status pending --limit 10");
        let response_ptr = unsafe { logbook_command(search_cmd.as_ptr(), ptr::null()) };
        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        
        assert_eq!(json["success"], true);
        assert!(json["data"].is_array());

        unsafe { logbook_response_free(response_ptr) };
    }

    #[test]
    fn test_logbook_command_search_with_order() {
        // Search with ordering
        let search_cmd = c_string("search transaction --limit 5 --order-by amount --order DESC");
        let response_ptr = unsafe { logbook_command(search_cmd.as_ptr(), ptr::null()) };
        let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
        let json: Value = serde_json::from_str(response).unwrap();
        
        assert_eq!(json["success"], true);
        assert!(json["data"].is_array());

        unsafe { logbook_response_free(response_ptr) };
    }
}
