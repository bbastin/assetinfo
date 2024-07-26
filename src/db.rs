// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{error::Error, fs::File, io::BufReader, path::PathBuf};

use eolmon::program::Program;

pub struct Database {
    pub path: PathBuf,
    pub supported_programs: Vec<Program>,
}

impl Database {
    pub fn load(path: PathBuf) -> Result<Database, Box<dyn Error>> {
        let mut supported_programs: Vec<Program> = Vec::default();

        for entry in path
            .read_dir()
            .expect("Could not read database directory")
            .flatten()
        {
            if !entry.file_type()?.is_file() {
                continue;
            }

            if !str::ends_with(
                entry
                    .file_name()
                    .to_str()
                    .expect("Invalid filename in database"),
                ".json",
            ) {
                continue;
            }

            let file = File::open(entry.path())?;
            let reader = BufReader::new(file);
            supported_programs.push(serde_json::from_reader(reader)?);
        }

        Ok(Database {
            path,
            supported_programs,
        })
    }

    pub fn get(&self, name: &str) -> Option<Program> {
        for p in &self.supported_programs {
            if p.id != name && p.title.to_lowercase() != name.to_lowercase() {
                continue;
            }

            return Some(p.clone());
        }

        None
    }
}
