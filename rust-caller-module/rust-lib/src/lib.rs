//! Rust caller module — demonstrates inter-module IPC via logos-rust-sdk.
//!
//! Uses LogosModuleSDK to call rust_provider_module methods through the
//! Logos IPC stack (Qt Remote Objects). The SDK handles parameter
//! serialization, CString management, and memory cleanup internally.
//!
//! logos_sdk_* symbols are resolved at final link time when CMake links
//! this staticlib against liblogos_module_client.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use logos_rust_sdk::LogosModuleSDK;

/// Call rust_provider_module.add(a, b) via IPC synchronously.
#[no_mangle]
pub extern "C" fn rust_caller_call_add(a: i64, b: i64) -> i64 {
    let sdk = LogosModuleSDK::new();
    let provider = sdk.plugin("rust_provider_module");
    match provider.call_sync("add", &[a, b]) {
        Ok(r) if r.success => r.message.parse::<i64>().unwrap_or(-1),
        Ok(r) => {
            eprintln!("rust_caller: add() failed: {}", r.message);
            -1
        }
        Err(e) => {
            eprintln!("rust_caller: add() IPC error: {}", e);
            -1
        }
    }
}

/// Call rust_provider_module.multiply(a, b) via IPC synchronously.
#[no_mangle]
pub extern "C" fn rust_caller_call_multiply(a: i64, b: i64) -> i64 {
    let sdk = LogosModuleSDK::new();
    let provider = sdk.plugin("rust_provider_module");
    match provider.call_sync("multiply", &[a, b]) {
        Ok(r) if r.success => r.message.parse::<i64>().unwrap_or(-1),
        Ok(r) => {
            eprintln!("rust_caller: multiply() failed: {}", r.message);
            -1
        }
        Err(e) => {
            eprintln!("rust_caller: multiply() IPC error: {}", e);
            -1
        }
    }
}

/// Call rust_provider_module.greet(name) via IPC synchronously.
/// Returns a heap-allocated C string that must be freed with rust_caller_free_string().
#[no_mangle]
pub extern "C" fn rust_caller_call_greet(name: *const c_char) -> *mut c_char {
    let name_str = if name.is_null() {
        "World".to_string()
    } else {
        unsafe { CStr::from_ptr(name) }
            .to_str()
            .unwrap_or("World")
            .to_string()
    };

    let sdk = LogosModuleSDK::new();
    let provider = sdk.plugin("rust_provider_module");
    let greeting = match provider.call_sync("greet", &[name_str.as_str()]) {
        Ok(r) if r.success => r.message,
        Ok(r) => format!("(greet failed: {})", r.message),
        Err(e) => format!("(IPC error: {})", e),
    };

    CString::new(greeting)
        .unwrap_or_else(|_| CString::new("(encoding error)").unwrap())
        .into_raw()
}

/// Free a string returned by rust_caller_call_greet().
#[no_mangle]
pub extern "C" fn rust_caller_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

/// Return the name of the provider module this caller communicates with.
/// Static string — do NOT free.
#[no_mangle]
pub extern "C" fn rust_caller_provider_name() -> *const c_char {
    b"rust_provider_module\0".as_ptr() as *const c_char
}
