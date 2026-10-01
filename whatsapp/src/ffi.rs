//! FFI bindings to the Go whatsmeow wrapper.

use core::ffi::{c_char, c_int};
use core::slice;

unsafe extern "C" {
    pub fn wa_pair_phone(
        phone: *const c_char,
        size: *mut c_int,
        success: *mut bool,
    ) -> *mut c_char;
    pub fn wa_needs_pairing() -> bool;
    pub fn wa_init_client(size: *mut c_int) -> *mut c_char;
    pub fn wa_is_synced() -> bool;
}

/// Converts a go static string to a Rust static string.
pub fn static_str(msg: *mut c_char, size: *mut c_int) -> &'static str {
    let data = msg.cast::<u8>();
    // SAFETY: shouldn't be null.
    #[expect(clippy::cast_sign_loss, clippy::as_conversions, reason = "u32")]
    let len = unsafe { *size as usize };
    // SAFETY: data is non null and valid for len chars.
    let sli = unsafe { slice::from_raw_parts(data, len) };
    // SAFETY: no UTF-8
    unsafe { str::from_utf8_unchecked(sli) }
}
