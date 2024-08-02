// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use async_compression::tokio::bufread::ZstdDecoder;
use log::info;
use tar::Archive;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::program::Program;

pub struct Database {
    pub path: PathBuf,
    pub supported_programs: Vec<Program>,
}

impl Database {
    pub fn load(path: &Path) -> Result<Database, Box<dyn Error>> {
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
            let reader = std::io::BufReader::new(file);
            supported_programs.push(serde_json::from_reader(reader)?);
        }

        Ok(Database {
            path: path.to_path_buf(),
            supported_programs,
        })
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<Program> {
        for program in &self.supported_programs {
            if program.info.id != name && program.info.title.to_lowercase() != name.to_lowercase() {
                continue;
            }

            return Some(program.clone());
        }

        None
    }

    pub fn check_update() {}

    pub async fn download_update(
        download_location: &str,
        download_dir: &Path,
    ) -> Result<PathBuf, Box<dyn Error>> {
        info!("Downloading new database '{download_location}'");

        let response = reqwest::get(download_location).await?;

        let filename = response
            .url()
            .path_segments()
            .and_then(Iterator::last)
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap();

        let filename = download_dir.join(filename);
        info!("Saving new database at '{}'", filename.to_str().unwrap());

        let mut dest = File::create(filename.clone())?;
        let bytes = response.bytes().await.unwrap();

        info!("File size: {}", bytes.len());

        // let raw_bytes = bytes.to_vec();

        let mut cursor = std::io::Cursor::new(bytes);

        // let hash = sha256::digest(raw_bytes);
        // assert_eq!(
        //     hash.as_str(),
        //     PathBuf::from(filename.file_stem().unwrap())
        //         .file_stem()
        //         .unwrap()
        //         .to_str()
        //         .unwrap()
        // );

        std::io::copy(&mut cursor, &mut dest)?;

        Ok(filename)
    }

    pub async fn install_update(
        update_file: &Path,
        update_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let decompressed_file = update_dir.join(update_file.file_stem().unwrap());

        Self::decompress_update(update_file, &decompressed_file).await?;

        Self::extract_update(&decompressed_file, update_dir)?;

        Ok(())
    }

    async fn decompress_update(
        compressed_file: &Path,
        decompressed_file: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let input = tokio::fs::File::open(compressed_file).await.unwrap();
        let output = tokio::fs::File::create(decompressed_file).await.unwrap();

        let mut reader = ZstdDecoder::new(tokio::io::BufReader::new(input));
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let mut output = tokio::io::BufWriter::new(output);

        Ok(output.write_all(&data).await?)
    }

    fn extract_update(update_file: &Path, update_dir: &Path) -> Result<(), Box<dyn Error>> {
        let mut ar = Archive::new(File::open(update_file).unwrap());

        Ok(ar.unpack(update_dir)?)
    }
}

#[cfg(test)]
mod tests {

    use crate::program::ProgramInfo;
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

        let db = Database::load(tmp_dir.path());

        assert!(db.is_ok());

        let db = db.unwrap();
        assert_eq!(db.supported_programs.len(), 1);
        assert_eq!(db.get(&testprogram.info.id), Some(testprogram));

        fs::remove_file(file_path).expect("Could not delete tmpfile");
    }

    #[tokio::test]
    #[ignore = "Needs network access (see integration tests)"]
    async fn download() {
        let tmp_dir = TempDir::new().expect("Could not create tmpdir");

        let update_file = Database::download_update("https://db.assetinfo.de/d45ab56217ea96762255f6f8840c4625ed5a025760169038f5aa2454c109cd26.tar.zstd", tmp_dir.path()).await.expect("Download failed");

        Database::install_update(&update_file, tmp_dir.path())
            .await
            .expect("Installation failed");
    }
}
