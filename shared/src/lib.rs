//! Shared traits and functions accross the mailbox applications.

#![allow(unused_features, reason = "bug")]
#![feature(stmt_expr_attributes)]

/// Loads and edits config.
mod config;

extern crate alloc;
use alloc::sync::Arc;
pub use std::sync::Mutex as StdMutex;

pub use config::{Config, EmailConfig, LoadError, SaveError};
pub use tokio::sync::Mutex as TokioMutex;

/// returns the default value for a type.
#[macro_export]
macro_rules! def {
    () => {
        Default::default()
    };
}

/// logs the work being done by the app.
#[macro_export]
macro_rules! log {
    ($($arg:expr),*) => {{
        eprintln!("\x1b[38;2;10;132;255m{}\x1b[0m", format!($($arg),*))
    }};
}

/// helper to create error messages of the right type.
#[macro_export]
macro_rules! errmsg {
    ($str:literal, $details:ident) => {{
        #[cfg(debug_assertions)]
        let msg = format!("{}: {}", $str, $details);
        #[cfg(not(debug_assertions))]
        let msg = $str;
        $crate::log!("{}: {}", $str, $details);
        msg
    }};
    ($str:expr) => {{
        #[cfg(debug_assertions)]
        let msg = format!("{}", $str);
        #[cfg(not(debug_assertions))]
        let msg = $str;
        $crate::log!("{}", $str);
        msg
    }};
}

/// helper to create error enumerations.
#[macro_export]
macro_rules! error {
    ($name:ident $(< $x:ident >)? : $($variant:ident $value:ty: $txt:literal,)*) => {
        #[expect(missing_docs, reason="name explicit enough")]
        #[derive(Debug)]
        pub enum $name$(< $x >)? {
            $($variant($value)),*
        }

        impl$(< $x >)? $name$(< $x >)? {
            /// Returns a short message corresponding to the error.
            pub fn display(&self) -> $crate::ErrStr {
                match self {
                    $(
                        #[cfg(debug_assertions)]
                        Self::$variant(msg) => format!("{}: {msg:?}", $txt),
                        #[cfg(not(debug_assertions))]
                        Self::$variant(_) => $txt,
                    )*
                }
            }
        }
    };
}

/// locks a mutex and unpoisons the error if poisoned.
#[macro_export]
macro_rules! lock {
    ($x:expr) => {
        $x.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    };
}

/// helper to create error enumerations.
#[macro_export]
macro_rules! display_err {
    ($msg:expr) => {{
        #[cfg(debug_assertions)]
        let msg = &$msg;
        #[cfg(not(debug_assertions))]
        let msg = $msg;
        msg
    }};
}

/// Arc mutex shorthand.
pub type ArMx<T> = Arc<StdMutex<T>>;

/// Async arc mutex shorthand.
pub type TokioArMx<T> = Arc<TokioMutex<T>>;

/// ,Type that contains the message to be displayed for this error.
#[cfg(debug_assertions)]
pub type ErrStr = String;
/// ,Type that contains the message to be displayed for this error.
#[cfg(not(debug_assertions))]
pub type ErrStr = &'static str;
