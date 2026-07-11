//! Shared traits and functions accross the mailbox applications.

/// Loads and edits config.
mod config;

extern crate alloc;
use alloc::sync::Arc;
use std::sync::Mutex;

pub use config::{Config, EmailConfig, LoadError, SaveError};

/// locks a mutex and unpoisons the error if poisoned.
#[macro_export]
macro_rules! lock {
    ($x:expr) => {
        $x.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    };
}

/// Arc mutex shorthand.
pub type ArMx<T> = Arc<Mutex<T>>;
