//! Whatsapp crate.
//!
//! Communicate with the `WhatsApp` client via FFI
//! to a Go whatsmeow wrapper.

mod ffi;

use core::ffi::{CStr, c_int};
use core::ptr;

use crate::ffi::{
    static_str, wa_init_client, wa_is_synced, wa_needs_pairing, wa_pair_phone
};

/// `WhatsApp` provider handler.
#[derive(Copy, Clone, Debug)]
pub struct Whatsapp(());

impl Whatsapp {
    /// Returns whether the initial sync has completed.
    #[must_use]
    pub fn is_synced(&self) -> bool {
        // SAFETY: FFI.
        unsafe { wa_is_synced() }
    }

    /// Returns whether we need to pair to a device or if it is already paired.
    #[must_use]
    pub fn needs_pairing(&self) -> bool {
        // SAFETY: FFI.
        unsafe { wa_needs_pairing() }
    }

    /// Initialise the whatsapp client.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection failed to be established.
    pub fn new() -> Result<Self, &'static str> {
        let size = ptr::null_mut::<c_int>();
        // SAFETY: FFI.
        let err = unsafe { wa_init_client(size) };

        if err.is_null() { Ok(Self(())) } else { Err(static_str(err, size)) }
    }

    /// Gets a pairing code from a phone number.
    ///
    /// # Errors
    ///
    /// Returns an error if the whatsmeow connection fails.
    pub fn pair_phone(&self, phone: &str) -> Result<String, &'static str> {
        let c_phone = phone.as_bytes().as_ptr().cast::<i8>();
        let success = ptr::null_mut::<bool>();
        let size = ptr::null_mut::<c_int>();
        // SAFETY: FFI
        let c_res = unsafe { wa_pair_phone(c_phone, size, success) };
        // SAFETY: never returns NULL
        let res =
            unsafe { CStr::from_ptr(c_res) }.to_string_lossy().to_string();
        // SAFETY: initialised by FFI
        if unsafe { *success } { Ok(res) } else { Err(static_str(c_res, size)) }
    }

    /// Validates a `WhatsApp` phone number.
    ///
    /// - Must not start with `+` or `0`
    /// - Must be at least 7 digits (shortest international format)
    /// - Must contain only ASCII digits
    ///
    /// # Errors
    ///
    /// Returns an error message if the phone is invalid.
    pub fn validate_phone(&self, phone: &str) -> Result<(), &'static str> {
        Err(if phone.is_empty() {
            "Missing phone number"
        } else if phone.starts_with('+') {
            "Phone must not start with +, use country code without it"
        } else if phone.starts_with('0') {
            "Phone must not start with 0, use country code instead"
        } else if phone.len() < 7 {
            "Phone too short, include country code"
        } else if !phone.chars().all(|ch| ch.is_ascii_digit()) {
            "Phone must contain only digits"
        } else {
            return Ok(());
        })
    }
}
