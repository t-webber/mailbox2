extern crate alloc;
use alloc::sync::Arc;
use std::collections::HashSet;
use std::fs::{read, write};
use std::path::PathBuf;

use color_eyre::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize, Serializer};

/// Configuration for one email provider.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EmailConfig {
    /// Domain of the email (server url).
    #[serde(deserialize_with = "deser_rc", serialize_with = "ser_rc")]
    domain: Arc<str>,
    /// Password or token.
    #[serde(deserialize_with = "deser_rc", serialize_with = "ser_rc")]
    password: Arc<str>,
    /// Port to hit (usually 993 for IMAPS).
    port: u16,
    /// Username (usually the email).
    #[serde(deserialize_with = "deser_rc", serialize_with = "ser_rc")]
    user: Arc<str>,
}

impl EmailConfig {
    /// Creates a config from these values.
    #[must_use]
    pub const fn new(
        user: Arc<str>,
        password: Arc<str>,
        domain: Arc<str>,
        port: u16,
    ) -> Self {
        Self { domain, password, port, user }
    }

    /// Returns the configuration values.
    #[must_use]
    pub fn values(&self) -> (&str, &str, &str, u16) {
        (&self.user, &self.password, &self.domain, self.port)
    }
}

/// Configuration to be saved and loaded.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Config {
    /// List of email providers.
    emails: HashSet<EmailConfig>,
}

impl Config {
    /// Adds a new email configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if it fails to save the configuration.
    pub fn add_email_config(&mut self, email: EmailConfig) -> Result<()> {
        self.emails.insert(email);
        self.save()
    }

    /// Returns the first email configuration, if there is one.
    #[must_use]
    pub fn as_first_email_config(&self) -> Option<&EmailConfig> {
        self.emails.iter().next()
    }

    /// Returns the list of email configurations.
    #[must_use]
    pub fn into_email_cfgs(self) -> HashSet<EmailConfig> {
        self.emails
    }

    /// Loads the configuration from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is in an invalid format.
    pub fn load() -> Result<Self> {
        let path = Self::path();
        if path.exists() {
            Ok(postcard::from_bytes(&read(path)?)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Returns the path to the configuration file.
    fn path() -> PathBuf {
        config_dir().unwrap_or_default().join(".mailbox")
    }

    /// Saves the config.
    fn save(&self) -> Result<()> {
        Ok(write(Self::path(), postcard::to_allocvec(self)?)?)
    }
}

/// Deserialises a [`Arc<str>`].
fn deser_rc<'de, D>(deser: D) -> Result<Arc<str>, D::Error>
where D: serde::Deserializer<'de> {
    Ok(Arc::from(String::deserialize(deser)?))
}

/// Serialises a [`Arc<str>`].
fn ser_rc<S>(data: &Arc<str>, ser: S) -> Result<S::Ok, S::Error>
where S: Serializer {
    ser.serialize_str(data)
}
