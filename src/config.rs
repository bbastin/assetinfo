// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Deserialize, Serialize)]
pub struct DatabaseConfig {
    path: PathBuf,
    update_url: String,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    log_level: Option<LogLevel>,
    database: DatabaseConfig,
}

impl Config {
    pub fn load(config_file_path: PathBuf) -> Result<Config, Box<dyn Error>> {
        let file_content = {
            let file = File::open(config_file_path)?;
            let mut reader = BufReader::new(file);
            let mut buf = String::default();
            let _size = reader.read_to_string(&mut buf)?;
            buf
        };

        Ok(toml::from_str(&file_content)?)
    }

    pub fn database_folder(&self) -> &Path {
        &self.database.path
    }

    pub fn update_url(&self) -> &str {
        &self.database.update_url
    }

    pub fn log_level(&self) -> Option<log::Level> {
        match self.log_level {
            Some(LogLevel::Error) => Some(log::Level::Error),
            Some(LogLevel::Warn) => Some(log::Level::Warn),
            Some(LogLevel::Info) => Some(log::Level::Info),
            Some(LogLevel::Debug) => Some(log::Level::Debug),
            Some(LogLevel::Trace) => Some(log::Level::Trace),
            None => None,
        }
    }
}
