// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    error::Error,
    fs::File,
    io::BufReader,
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
pub struct Config {
    database_folder: PathBuf,
    update_url: String,
    log_level: Option<LogLevel>,
}

impl Config {
    pub fn load(config_file_path: PathBuf) -> Result<Config, Box<dyn Error>> {
        let file = File::open(config_file_path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn database_folder(&self) -> &Path {
        &self.database_folder
    }

    pub fn update_url(&self) -> &str {
        &self.update_url
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
