use serde::{Deserialize, Serialize};

use std::env;
use std::fs;
use std::io::ErrorKind::NotFound;

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Language used when printing pokemon names. Possible values include
    /// en, fr, de, ja, ja_ro, zh_hans, zh_hant
    pub language: String,
    /// The probability to display a shiny pokemon with the random command
    pub shiny_rate: f64,

    #[serde(skip)]
    pub program_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        // Should be safe to unwrap here, otherwise something is seriously wrong
        let bin_path = env::current_exe().unwrap();
        let mut program_dir = bin_path.parent().unwrap();
        while !program_dir.ends_with(BINARY_NAME) {
            program_dir = program_dir.parent().unwrap();
        }
        let program_dir = program_dir
            .to_str()
            .expect("Could not convert current directory path to unicode");

        Self {
            language: "en".to_string(),
            shiny_rate: 1.0 / 128.0,
            program_dir: program_dir.to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, &'static str> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir.join(BINARY_NAME),
            None => return Err("Failed to get config directory"),
        };

        let config_file = config_dir.join("config.toml");
        let config = match fs::read_to_string(&config_file) {
            Ok(c) => toml::from_str(&c).expect("Failed to parse toml in configuration file"),

            // Create default config file if it doesn't exist
            Err(ref e) if e.kind() == NotFound => {
                let config = Config::default();
                let toml =
                    toml::to_string_pretty(&config).expect("Failed to convert config to toml");

                fs::create_dir_all(config_dir).expect("Failed to create config directory");
                fs::write(&config_file, toml).expect("Failed to write config file");
                config
            }
            Err(_) => return Err("Failed to load configuration file"),
        };

        Ok(config)
    }
}
