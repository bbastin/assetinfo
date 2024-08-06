// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use sha256::try_digest;

use crate::hash_database::{HashDatabase, VersionedProgramInfo};

fn calculate_hash(file: &Path) -> Result<String, Box<dyn Error>> {
    Ok(try_digest(file)?)
}

fn scan_folder(folder: &Path) -> Result<Vec<(PathBuf, String)>, Box<dyn Error>> {
    let mut file_hashes = Vec::default();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            file_hashes.push((path, calculate_hash(&entry.path())?));
        }
    }

    Ok(file_hashes)
}

#[derive(Debug)]
pub struct FileScanResult {
    pub file_path: PathBuf,
    pub file_hash: String,
    pub program_info: Option<VersionedProgramInfo>,
}

pub fn scan(
    paths: Vec<PathBuf>,
    hash_databases: &[impl HashDatabase],
) -> Result<Vec<FileScanResult>, Box<dyn Error>> {
    let mut scan_results = Vec::default();

    for path in paths {
        let file_hashes = scan_folder(&path)?;

        for (file_path, file_hash) in file_hashes {
            let mut scan_result = Option::default();

            for database in hash_databases {
                if let Some(program_info) = database.get(&file_hash)? {
                    scan_result = Some(FileScanResult {
                        file_path: file_path.clone(),
                        file_hash: file_hash.clone(),
                        program_info: Some(program_info),
                    });
                    break;
                }
            }
            if let Some(scan_result) = scan_result {
                scan_results.push(scan_result);
            } else {
                scan_results.push(FileScanResult {
                    file_path,
                    file_hash,
                    program_info: None,
                });
            }
        }
    }

    Ok(scan_results)
}
