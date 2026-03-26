//! Test FFI functions directly from the library

use logbook::ffi::{logbook_init, logbook_free, logbook_request, logbook_handle_json, logbook_version};
use std::ffi::CString;
use std::ptr;

fn c_string(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn c_str_to_string(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap().to_string()
}

fn main() {
    println!("Testing logbook FFI...\n");

    // Test 1: Version
    println!("=== Test 1: Version ===");
    let version = logbook_version();
    println!("Version: {}", c_str_to_string(version));
    println!();

    // Test 2: Handle-based API
    println!("=== Test 2: Handle-based API ===");
    unsafe {
        let handle = logbook_init(ptr::null());
        
        if handle.is_null() {
            println!("ERROR: Failed to initialize logbook");
            return;
        }
        println!("Logbook initialized successfully");

        // Create a transaction
        let create_req = c_string(r#"{"tool":"create_transaction","args":{"amount":25.50,"kind":"ffi_test","description":"Rust FFI test"}}"#);
        println!("Creating transaction...");
        let response = logbook_request(handle, create_req.as_ptr());
        if !response.is_null() {
            println!("Response: {}", c_str_to_string(response));
        } else {
            println!("ERROR: Request failed");
        }

        // List transactions
        let list_req = c_string(r#"{"tool":"list_transactions","args":{}}"#);
        println!("\nListing transactions...");
        let response = logbook_request(handle, list_req.as_ptr());
        if !response.is_null() {
            println!("Response: {}", c_str_to_string(response));
        }

        logbook_free(handle);
        println!("\nLogbook freed successfully");
    }
    println!();

    // Test 3: One-shot JSON API
    println!("=== Test 3: One-shot JSON API ===");
    unsafe {
        let json_req = c_string(r#"{"tool":"list_categories","args":{}}"#);
        println!("Request: list_categories");
        let response = logbook_handle_json(json_req.as_ptr(), ptr::null());
        if !response.is_null() {
            println!("Response: {}", c_str_to_string(response));
        } else {
            println!("ERROR: Request failed");
        }
    }
    println!();

    println!("=== All FFI tests completed! ===");
}
