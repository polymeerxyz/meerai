use serde::Deserialize;

/// Configuration for X (Twitter) integration
#[derive(Debug, Deserialize)]
pub struct XConfig {
    /// Cookie string for authentication
    pub cookie: String,
}
