// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{error::Error, fs::File, io::BufReader, path::PathBuf};

use assetinfo::program::Program;

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
            if p.info.id != name && p.info.title.to_lowercase() != name.to_lowercase() {
                continue;
            }

            return Some(p.clone());
        }

        None
    }
}

#[cfg(test)]
mod tests {

    use assetinfo::program::ProgramInfo;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test() {
        let tmp_dir = TempDir::new().expect("Could not create tmpdir");
        let file_path = tmp_dir.path().join("testprogram.json");
        let mut tmp_file = File::create(file_path.clone()).expect("Could not create tmpfile");

        let testprogram = Program {
            info: ProgramInfo {
                id: "testprogram".to_string(),
                title: "Testprogram".to_string(),
                endoflife_date_id: None,
            },
            binary: None,
            docker: None,
        };

        writeln!(tmp_file, "{}", serde_json::to_string(&testprogram).unwrap())
            .expect("Could not write to tmpfile");
        tmp_file.flush().expect("Could not flush tmpfile");
        drop(tmp_file);

        let db = Database::load(tmp_dir.path().to_path_buf());

        assert!(db.is_ok());

        let db = db.unwrap();
        assert_eq!(db.supported_programs.len(), 1);
        assert_eq!(db.get(&testprogram.info.id), Some(testprogram));

        fs::remove_file(file_path).expect("Could not delete tmpfile");
    }
}
