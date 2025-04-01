use std::{env, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Regex error: {0}")]
    RegexError(String),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] env::VarError),
}
