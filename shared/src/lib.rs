//! Shared traits and functions accross the mailbox applications.

/// Loads and edits config.
mod config;

extern crate alloc;
use alloc::sync::Arc;
pub use std::sync::Mutex as StdMutex;

pub use config::{Config, EmailConfig, LoadError, SaveError};
pub use tokio::sync::Mutex as TokioMutex;

/// logs the work being done by the app.
#[macro_export]
macro_rules! log {
    ($($arg:expr),*) => {{
        //         eprintln!($($arg),*)
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
            pub fn display(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => $txt,)*
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

/// Arc mutex shorthand.
pub type ArMx<T> = Arc<StdMutex<T>>;

/// Async arc mutex shorthand.
pub type TokioArMx<T> = Arc<TokioMutex<T>>;
