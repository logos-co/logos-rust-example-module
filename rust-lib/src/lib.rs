use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Add two integers.
#[no_mangle]
pub extern "C" fn rust_example_add(a: i64, b: i64) -> i64 {
    a + b
}

/// Multiply two integers.
#[no_mangle]
pub extern "C" fn rust_example_multiply(a: i64, b: i64) -> i64 {
    a.saturating_mul(b)
}

/// Compute n! (factorial). Returns -1 on overflow or negative input.
#[no_mangle]
pub extern "C" fn rust_example_factorial(n: i64) -> i64 {
    if n < 0 {
        return -1;
    }
    if n <= 1 {
        return 1;
    }
    let mut result: i64 = 1;
    for i in 2..=n {
        result = match result.checked_mul(i) {
            Some(v) => v,
            None => return -1,
        };
    }
    result
}

/// Compute the nth Fibonacci number. Returns -1 on overflow or negative input.
#[no_mangle]
pub extern "C" fn rust_example_fibonacci(n: i64) -> i64 {
    if n < 0 {
        return -1;
    }
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let (mut a, mut b) = (0i64, 1i64);
    for _ in 2..=n {
        let next = match a.checked_add(b) {
            Some(v) => v,
            None => return -1,
        };
        a = b;
        b = next;
    }
    b
}

/// Return 1 if n is prime, 0 otherwise. Negative numbers and 0/1 return 0.
#[no_mangle]
pub extern "C" fn rust_example_is_prime(n: i64) -> i64 {
    if n < 2 {
        return 0;
    }
    if n == 2 {
        return 1;
    }
    if n % 2 == 0 {
        return 0;
    }
    let mut i = 3i64;
    while i * i <= n {
        if n % i == 0 {
            return 0;
        }
        i += 2;
    }
    1
}

/// Greet the given name. Returns a static pointer to a greeting string.
/// The caller must NOT free the returned pointer.
#[no_mangle]
pub extern "C" fn rust_example_greet(name: *const c_char) -> *mut c_char {
    let name_str = if name.is_null() {
        "World".to_string()
    } else {
        unsafe { CStr::from_ptr(name) }
            .to_str()
            .unwrap_or("World")
            .to_string()
    };
    let greeting = format!("Hello, {}! (from Rust)", name_str);
    CString::new(greeting)
        .unwrap_or_else(|_| CString::new("Hello! (from Rust)").unwrap())
        .into_raw()
}

/// Free a string previously returned by rust_example_greet.
/// Calling this on any other pointer is undefined behaviour.
#[no_mangle]
pub extern "C" fn rust_example_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

/// Return the Rust library version string. The returned pointer is valid
/// for the lifetime of the process and must NOT be freed.
#[no_mangle]
pub extern "C" fn rust_example_version() -> *const c_char {
    b"1.0.0\0".as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(rust_example_add(2, 3), 5);
        assert_eq!(rust_example_add(-1, 1), 0);
        assert_eq!(rust_example_add(0, 0), 0);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(rust_example_multiply(3, 4), 12);
        assert_eq!(rust_example_multiply(-2, 5), -10);
        assert_eq!(rust_example_multiply(0, 100), 0);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(rust_example_factorial(0), 1);
        assert_eq!(rust_example_factorial(1), 1);
        assert_eq!(rust_example_factorial(5), 120);
        assert_eq!(rust_example_factorial(10), 3628800);
        assert_eq!(rust_example_factorial(-1), -1);
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(rust_example_fibonacci(0), 0);
        assert_eq!(rust_example_fibonacci(1), 1);
        assert_eq!(rust_example_fibonacci(10), 55);
        assert_eq!(rust_example_fibonacci(-1), -1);
    }

    #[test]
    fn test_is_prime() {
        assert_eq!(rust_example_is_prime(2), 1);
        assert_eq!(rust_example_is_prime(7), 1);
        assert_eq!(rust_example_is_prime(4), 0);
        assert_eq!(rust_example_is_prime(1), 0);
        assert_eq!(rust_example_is_prime(-5), 0);
    }

    #[test]
    fn test_version() {
        let v = unsafe { CStr::from_ptr(rust_example_version()) };
        assert_eq!(v.to_str().unwrap(), "1.0.0");
    }
}
