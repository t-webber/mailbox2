//! FFI bindings to the Go whatsmeow wrapper.

use core::ffi::c_char;

unsafe extern "C" {
    pub fn wa_pair_phone(phone: *const c_char) -> *mut c_char;
    pub fn wa_needs_pairing() -> bool;
    pub fn wa_init_client() -> *mut c_char;
    pub fn wa_is_synced() -> bool;
}
