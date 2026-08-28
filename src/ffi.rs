//! C FFI exports for Foundation.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe fn read_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(str::to_owned)
}

/// The framework version as a static C string.
#[no_mangle]
pub extern "C" fn tontoo_foundation_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Current time as a Unix timestamp in seconds.
#[no_mangle]
pub extern "C" fn tontoo_foundation_date_now() -> i64 {
    crate::date::Date::now().timestamp()
}

/// Shift a Unix timestamp by `days` days.
#[no_mangle]
pub extern "C" fn tontoo_foundation_date_add_days(secs: i64, days: i64) -> i64 {
    crate::date::Date::from_timestamp(secs)
        .adding_days(days)
        .timestamp()
}

/// Whether timestamp `a` is before timestamp `b`.
#[no_mangle]
pub extern "C" fn tontoo_foundation_date_is_before(a: i64, b: i64) -> i32 {
    let (a, b) = (
        crate::date::Date::from_timestamp(a),
        crate::date::Date::from_timestamp(b),
    );
    a.is_before(&b) as i32
}

/// Read a string from the standard defaults store. Returns null when the
/// key is unset.
///
/// # Safety
///
/// `key` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_foundation_defaults_get_string(
    key: *const c_char,
) -> *mut c_char {
    let Some(key) = read_str(key) else {
        return std::ptr::null_mut();
    };
    let mut defaults = crate::userdefaults::UserDefaults::standard();
    if defaults.init().is_err() {
        return std::ptr::null_mut();
    }
    match defaults.string(&key).map(str::to_owned) {
        Some(value) => CString::new(value).unwrap_or_default().into_raw(),
        None => std::ptr::null_mut(),
    }
}

/// Write a string to the standard defaults store. Returns 0 on success.
///
/// # Safety
///
/// `key` and `value` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_foundation_defaults_set_string(
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    let (Some(key), Some(value)) = (read_str(key), read_str(value)) else {
        return -1;
    };
    let mut defaults = crate::userdefaults::UserDefaults::standard();
    if defaults.init().is_err() {
        return -2;
    }
    defaults.set_string(&key, &value);
    if defaults.save().is_err() {
        return -3;
    }
    0
}

/// Read an integer from the standard defaults store.
///
/// # Safety
///
/// `key` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_foundation_defaults_get_int(key: *const c_char) -> i64 {
    let Some(key) = read_str(key) else {
        return 0;
    };
    let mut defaults = crate::userdefaults::UserDefaults::standard();
    if defaults.init().is_err() {
        return 0;
    }
    defaults.int(&key)
}

/// Write an integer to the standard defaults store. Returns 0 on success.
///
/// # Safety
///
/// `key` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_foundation_defaults_set_int(
    key: *const c_char,
    value: i64,
) -> i32 {
    let Some(key) = read_str(key) else {
        return -1;
    };
    let mut defaults = crate::userdefaults::UserDefaults::standard();
    if defaults.init().is_err() {
        return -2;
    }
    defaults.set_int(&key, value);
    if defaults.save().is_err() {
        return -3;
    }
    0
}

/// Read a boolean from the standard defaults store.
///
/// # Safety
///
/// `key` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_foundation_defaults_get_bool(key: *const c_char) -> i32 {
    let Some(key) = read_str(key) else {
        return 0;
    };
    let mut defaults = crate::userdefaults::UserDefaults::standard();
    if defaults.init().is_err() {
        return 0;
    }
    defaults.bool(&key) as i32
}

/// Write a boolean to the standard defaults store. Returns 0 on success.
///
/// # Safety
///
/// `key` must be NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn tontoo_foundation_defaults_set_bool(
    key: *const c_char,
    value: i32,
) -> i32 {
    let Some(key) = read_str(key) else {
        return -1;
    };
    let mut defaults = crate::userdefaults::UserDefaults::standard();
    if defaults.init().is_err() {
        return -2;
    }
    defaults.set_bool(&key, value != 0);
    if defaults.save().is_err() {
        return -3;
    }
    0
}

/// Free a string returned by this library.
///
/// # Safety
///
/// `s` must be a pointer returned by this API or null.
#[no_mangle]
pub unsafe extern "C" fn tontoo_foundation_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
