mod error;

pub use error::ConfigError;
use regex::Regex;
use serde::de::DeserializeOwned;
use std::{env, fs, path::Path};

/// Load configuration from config.yml file with environment variable expansion                                                                                                                                            
pub fn load_config<T: DeserializeOwned>() -> Result<T, ConfigError> {
    let config_path = Path::new("config.yml");
    let content = fs::read_to_string(config_path)?;
    let expanded = expand_env_vars(&content)?;
    Ok(serde_yaml::from_str(&expanded)?)
}

fn expand_env_vars(content: &str) -> Result<String, ConfigError> {
    let re = Regex::new(r"\$\{([^}]+)\}")
        .map_err(|e| ConfigError::RegexError(format!("Failed to compile env var regex: {}", e)))?;

    let result = re.replace_all(content, |caps: &regex::Captures| {
        let var = &caps[1];
        match var.split_once('|') {
            Some((var_name, default)) => env::var(var_name).unwrap_or_else(|_| default.to_string()),
            None => env::var(var)
                .inspect_err(|_| {
                    eprintln!(
                        "Warning: Environment variable {} not found - using empty string",
                        var
                    );
                })
                .unwrap_or_default(),
        }
    });

    Ok(result.to_string())
}
