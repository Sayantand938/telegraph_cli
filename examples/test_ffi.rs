//! Test FFI functions directly from the library

use tx_tracker::ffi::{tracker_init, tracker_free, tracker_request, tracker_handle_json, tracker_version};
use std::ffi::CString;
use std::ptr;

fn c_string(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn c_str_to_string(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap().to_string()
}

fn main() {
    println!("Testing tx_tracker FFI...\n");

    // Test 1: Version
    println!("=== Test 1: Version ===");
    unsafe {
        let version = tracker_version();
        println!("Version: {}", c_str_to_string(version));
    }
    println!();

    // Test 2: Handle-based API
    println!("=== Test 2: Handle-based API ===");
    unsafe {
        let handle = tracker_init(ptr::null());
        
        if handle.is_null() {
            println!("ERROR: Failed to initialize tracker");
            return;
        }
        println!("Tracker initialized successfully");

        // Create a transaction
        let create_req = c_string(r#"{"tool":"create_transaction","args":{"amount":25.50,"kind":"ffi_test","description":"Rust FFI test"}}"#);
        println!("Creating transaction...");
        let response = tracker_request(handle, create_req.as_ptr());
        if !response.is_null() {
            println!("Response: {}", c_str_to_string(response));
        } else {
            println!("ERROR: Request failed");
        }

        // List transactions
        let list_req = c_string(r#"{"tool":"list_transactions","args":{}}"#);
        println!("\nListing transactions...");
        let response = tracker_request(handle, list_req.as_ptr());
        if !response.is_null() {
            println!("Response: {}", c_str_to_string(response));
        }

        tracker_free(handle);
        println!("\nTracker freed successfully");
    }
    println!();

    // Test 3: One-shot JSON API
    println!("=== Test 3: One-shot JSON API ===");
    unsafe {
        let json_req = c_string(r#"{"tool":"list_categories","args":{}}"#);
        println!("Request: list_categories");
        let response = tracker_handle_json(json_req.as_ptr(), ptr::null());
        if !response.is_null() {
            println!("Response: {}", c_str_to_string(response));
        } else {
            println!("ERROR: Request failed");
        }
    }
    println!();

    println!("=== All FFI tests completed! ===");
}
