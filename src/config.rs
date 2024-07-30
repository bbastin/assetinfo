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
pub struct Config {
    database_folder: PathBuf,
    update_url: String,
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
}
