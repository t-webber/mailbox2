//! Whatsapp crate.
//!
//! Communicate with the `WhatsApp` client via FFI
//! to a Go whatsmeow wrapper.

#![allow(clippy::restriction)]

mod ffi;
use core::ffi::CStr;

/// Get a pairing code from a phone number.
///
/// # Errors
///
/// Returns an error if the whatsmeow connection fails.
pub fn pair_phone(phone: &str) -> Result<String, String> {
    // SAFETY: FFI
    let c_res =
        unsafe { ffi::wa_pair_phone(phone.as_bytes().as_ptr().cast::<i8>()) };
    // SAFETY: FFI
    let res = unsafe { CStr::from_ptr(c_res) }.to_string_lossy().to_string();
    if res.len() == 9 { Ok(res) } else { Err(res) }
}
