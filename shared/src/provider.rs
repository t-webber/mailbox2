extern crate alloc;

use alloc::borrow::Cow;
use core::fmt::Debug;

use color_eyre::Result;

/// Generic provider trait.
///
/// Anything that is displayed in the app is a provider (emails, mautrixes,
/// etc.)
pub trait Provider: Sized {
    /// Type used for authentication.
    type Auth;
    /// Cf. [`Message`].
    type Message: Message;
    /// Cf. [`Room`].
    type Room: Room;

    /// Authenticates and establishes the connection.
    fn auth(config: &Self::Auth) -> impl Future<Output = Result<Self>>;

    /// Fetches the body of the room.
    fn get_messages(
        &mut self,
        room: &Self::Room,
    ) -> impl Future<Output = Result<Vec<Self::Message>>>;

    /// Returns the list of [`Self::Room`] for this [`Provider`].
    fn get_rooms(&mut self) -> impl Future<Output = Result<Vec<Self::Room>>>;
}

/// A room to display.
///
/// A room is a succession of messages between multiple people.
///
/// # Examples
///
/// - For emails, this is the chain of replies of an email.
/// - For messages, it's a thread with another person or group.
pub trait Room: Debug {
    /// Displays the entire content of the room.
    fn debug(&self) -> String;
    /// Returns the name of the room, or the person with whom this room is.
    fn name(&self) -> Cow<'_, str>;
    /// Quick overview of the room (last message, subject, etc.)
    fn overview(&self) -> Cow<'_, str>;
}

/// A message of a [`Room`].
pub trait Message: Debug {
    /// Displays the entire content of the room.
    fn debug(&self) -> String;
}
