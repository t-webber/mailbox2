//! FFI bindings to the Go whatsmeow wrapper.

use core::ffi::c_char;

unsafe extern "C" {
    pub fn wa_pair_phone(phone: *const c_char) -> *mut c_char;
}
