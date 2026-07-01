//! Shared traits and functions accross the mailbox applications.

use color_eyre::Result;

/// Generic provider trait.
///
/// Anything that is displayed in the app is a provider (emails, mautrixes,
/// etc.)
pub trait Provider: Sized {
    /// A room to display.
    ///
    /// A room is a succession of messages between multiple people.
    ///
    /// # Examples
    ///
    /// - For emails, this is the chain of replies of an email.
    /// - For messages, it's a thread with another person or group.
    type Room;

    /// Authenticates and establishes the connection.
    fn auth() -> impl Future<Output = Result<Self>>;

    /// Returns the list of [`Self::Room`] for this [`Provider`].
    fn get_rooms(&mut self) -> impl Future<Output = Result<Vec<Self::Room>>>;
}
