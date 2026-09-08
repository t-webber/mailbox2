//! FFI bindings to the Go whatsmeow wrapper.

use core::ffi::{CStr, c_char};

unsafe extern "C" {
    fn wa_pair_phone(phone: *const c_char) -> *mut c_char;
    fn wa_needs_pairing() -> bool;
    fn wa_init_client() -> *mut c_char;
    fn wa_is_synced() -> bool;
}

/// Gets a pairing code from a phone number.
///
/// # Errors
///
/// Returns an error if the whatsmeow connection fails.
pub fn pair_phone(phone: &str) -> Result<String, String> {
    // SAFETY: FFI
    let c_res =
        unsafe { wa_pair_phone(phone.as_bytes().as_ptr().cast::<i8>()) };
    // SAFETY: never returns NULL
    let res = unsafe { CStr::from_ptr(c_res) }.to_string_lossy().to_string();
    if res.len() == 9 { Ok(res) } else { Err(res) }
}

/// Returns whether we need to pair to a device or if it is already paired.
#[must_use]
pub fn needs_pairing() -> bool {
    // SAFETY: FFI.
    unsafe { wa_needs_pairing() }
}

/// Initialise the whatsapp client.
///
/// # Errors
///
/// Returns an error if the connection failed to be established.
pub fn initialise_client() -> Result<(), String> {
    // SAFETY: FFI.
    let err = unsafe { wa_init_client() };
    if err.is_null() {
        Ok(())
    } else {
        // SAFETY: checked for null
        Err(unsafe { CStr::from_ptr(err) }.to_string_lossy().to_string())
    }
}

/// Returns whether the initial sync has completed.
#[must_use]
pub fn is_synced() -> bool {
    // SAFETY: FFI.
    unsafe { wa_is_synced() }
}
