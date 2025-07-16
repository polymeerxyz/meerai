mod bluesky;
mod x;

pub use bluesky::BlueskyConfig;
use meerai_common::config;
use serde::Deserialize;
pub use x::XConfig;

/// Main application configuration                                                                                                                                                                                         
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub x: XConfig,
    pub bluesky: BlueskyConfig,
}

pub fn load_config() -> Result<Config, config::ConfigError> {
    config::load_config::<Config>()
}
