//! Whatsapp crate.
//!
//! Communicate with the `WhatsApp` client via FFI
//! to a Go whatsmeow wrapper.

pub mod ffi;

/// Validates a `WhatsApp` phone number.
///
/// - Must not start with `+` or `0`
/// - Must be at least 7 digits (shortest international format)
/// - Must contain only ASCII digits
///
/// # Errors
///
/// Returns an error message if the phone is invalid.
pub fn validate_phone(phone: &str) -> Result<(), &'static str> {
    if phone.starts_with('+') {
        return Err("Phone must not start with +, use country code without it");
    }
    if phone.starts_with('0') {
        return Err("Phone must not start with 0, use country code instead");
    }
    if phone.len() < 7 {
        return Err("Phone too short, include country code");
    }
    if !phone.chars().all(|ch| ch.is_ascii_digit()) {
        return Err("Phone must contain only digits");
    }
    Ok(())
}
