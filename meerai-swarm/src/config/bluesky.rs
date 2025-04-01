use serde::Deserialize;

/// Configuration for Bluesky integration
#[derive(Debug, Deserialize)]
pub struct BlueskyConfig {
    /// Account identifier (username)
    pub identifier: String,

    /// Account password
    pub password: String,
}
