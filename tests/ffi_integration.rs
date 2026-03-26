//! FFI Integration Tests
//! 
//! These tests verify that the FFI functions work correctly
//! after the DRY refactoring.

use std::ffi::{CStr, CString};
use std::ptr;
use logbook::ffi::{logbook_init, logbook_free, logbook_request, logbook_response_free, logbook_handle_json, logbook_version};

fn c_string(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[test]
fn test_ffi_version() {
    let version_ptr = unsafe { logbook_version() };
    assert!(!version_ptr.is_null());
    let version = unsafe { CStr::from_ptr(version_ptr) }.to_str().unwrap();
    assert_eq!(version, "0.1.0");
}

#[test]
fn test_ffi_init_and_free() {
    let db_path = c_string("test_ffi_init.db");
    let handle = unsafe { logbook_init(db_path.as_ptr()) };
    assert!(!handle.is_null());
    unsafe { logbook_free(handle) };
    let _ = std::fs::remove_file("test_ffi_init.db");
}

#[test]
fn test_ffi_handle_json_list_categories() {
    let request = c_string(r#"{"tool":"list_categories","args":{}}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    assert!(response.contains("category"));
    
    unsafe { logbook_response_free(response_ptr) };
}

#[test]
fn test_ffi_handle_json_create_transaction() {
    let request = c_string(r#"{"tool":"create_transaction","args":{"amount":99.99,"kind":"test","description":"FFI test"}}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    assert!(response.contains(r#""id":"#));
    
    unsafe { logbook_response_free(response_ptr) };
}

#[test]
fn test_ffi_handle_json_list_transactions() {
    let request = c_string(r#"{"tool":"list_transactions","args":{}}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    
    unsafe { logbook_response_free(response_ptr) };
}

#[test]
fn test_ffi_handle_json_create_activity() {
    let request = c_string(r#"{"tool":"create_activity","args":{"start_time":"09:00","stop_time":"10:00","description":"FFI activity test"}}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    assert!(response.contains(r#""id":"#));
    
    unsafe { logbook_response_free(response_ptr) };
}

#[test]
fn test_ffi_handle_json_create_todo() {
    let request = c_string(r#"{"tool":"create_todo","args":{"description":"FFI todo test","status":"pending"}}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    assert!(response.contains(r#""id":"#));
    
    unsafe { logbook_response_free(response_ptr) };
}

#[test]
fn test_ffi_handle_json_create_journal() {
    let request = c_string(r#"{"tool":"create_journal","args":{"content":"FFI journal entry test"}}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    assert!(response.contains(r#""id":"#));
    
    unsafe { logbook_response_free(response_ptr) };
}

#[test]
fn test_ffi_handle_with_persistent_handle() {
    let db_path = c_string("test_ffi_persistent.db");
    let handle = unsafe { logbook_init(db_path.as_ptr()) };
    assert!(!handle.is_null());
    
    // Create a transaction using the handle
    let create_request = c_string(r#"{"tool":"create_transaction","args":{"amount":50.0,"kind":"shopping","description":"Handle test"}}"#);
    let response_ptr = unsafe { logbook_request(handle, create_request.as_ptr()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    unsafe { logbook_response_free(response_ptr) };
    
    // List transactions using the same handle
    let list_request = c_string(r#"{"tool":"list_transactions","args":{}}"#);
    let response_ptr = unsafe { logbook_request(handle, list_request.as_ptr()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":true"#));
    unsafe { logbook_response_free(response_ptr) };
    
    unsafe { logbook_free(handle) };
    let _ = std::fs::remove_file("test_ffi_persistent.db");
}

#[test]
fn test_ffi_invalid_json() {
    let request = c_string(r#"{"tool": invalid}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":false"#));
    assert!(response.contains(r#""error":"#));
    
    unsafe { logbook_response_free(response_ptr) };
}

#[test]
fn test_ffi_unknown_tool() {
    let request = c_string(r#"{"tool":"unknown_tool","args":{}}"#);
    let response_ptr = unsafe { logbook_handle_json(request.as_ptr(), ptr::null()) };
    assert!(!response_ptr.is_null());
    
    let response = unsafe { CStr::from_ptr(response_ptr) }.to_str().unwrap();
    assert!(response.contains(r#""success":false"#));
    assert!(response.contains(r#""error":"#));
    
    unsafe { logbook_response_free(response_ptr) };
}
