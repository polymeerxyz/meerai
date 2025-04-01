mod bluesky;
mod x;

pub use bluesky::BlueskyConfig;
use meerai_common::config;
pub use x::XConfig;

/// Main application configuration                                                                                                                                                                                         
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub x: XConfig,
    pub bluesky: BlueskyConfig,
}

pub fn load_config() -> Result<Config, config::ConfigError> {
    config::load_config::<Config>()
}
