use std::{
    env,
    path::{Path, PathBuf},
};

use config::{Config, ConfigError};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::notify::NotifyType;

#[derive(Deserialize, Serialize)]
pub struct Setting {
    pub log_level: String,
    pub warning_threshold: f64,
    pub api_key: String,
    pub notifiers: Vec<NotifyType>,
    pub sleeptime: u64,
}

impl Setting {
    pub fn new(env_name: &str, project_dir: Option<ProjectDirs>) -> Result<Self, ConfigError> {
        let file_path = get_config_path(env_name, project_dir);

        let settings = Config::builder()
            .add_source(config::File::from(file_path))
            .add_source(config::Environment::with_prefix("FOREX_NOTIFY"))
            .build()?;

        let settings: Setting = settings.try_deserialize()?;
        if settings.notifiers.is_empty() {
            return Err(ConfigError::Message("No notifiers found".to_string()));
        }

        Ok(settings)
    }
}

fn get_config_path(env_name: &str, project_dir: Option<ProjectDirs>) -> PathBuf {
    // 1. Check the environment variable
    if let Ok(config_path) = env::var(env_name) {
        let path = Path::new(&config_path);
        if path.exists() {
            return path.to_path_buf();
        } else {
            panic!("Configuration file specified by FOREX_NOTIFY_CONFIG does not exist");
        }
    }

    // 2. Check the current directory
    let current_dir = Path::new(".");
    let config_in_current_dir = current_dir.join("config.toml");
    if config_in_current_dir.exists() {
        return config_in_current_dir;
    }

    // 3. Check the user configuration directory
    if let Some(proj_dirs) = project_dir {
        let user_config_dir = proj_dirs.config_dir().join("config.toml");
        if user_config_dir.exists() {
            return user_config_dir;
        }
    }

    panic!("No valid configuration file found");
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::notify::{
        ntfy::Ntfy,
        telegram::Telegram,
        webhook::{Method, Webhook},
    };

    use super::*;

    #[test]
    fn test_serialize_setting() {
        let setting = Setting {
            log_level: "info".to_string(),
            warning_threshold: 0.9980,
            api_key: "demo".to_string(),
            notifiers: vec![
                NotifyType::Telegram(Telegram::new("token", "chat_id")),
                NotifyType::Ntfy(Ntfy::new("url", Some("token"), Some("title"), Some(4))),
                NotifyType::Webhook(Webhook::new(
                    "http://example.com",
                    HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
                    Some(
                        "{
                        \"under_threshold\": {under_threshold},
                        \"rate\": {rate}
                    }"
                        .to_string(),
                    ),
                    Method::POST,
                )),
            ],
            sleeptime: 180,
        };

        let toml = toml::to_string(&setting).unwrap();
        // save to config.toml.example
        std::fs::write("config.toml.example", toml).unwrap();
    }
}
