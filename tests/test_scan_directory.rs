// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::HashMap, error::Error, fs::File, io::Write};

use assetinfo::{
    hash_database::{HashDatabase, VersionedProgramInfo},
    program::Version,
    scan::scan,
};
use tempfile::TempDir;

pub struct MockHashDatabase {
    hashes: HashMap<String, VersionedProgramInfo>,
}

impl HashDatabase for MockHashDatabase {
    fn get(&self, hash: &str) -> Result<Option<VersionedProgramInfo>, Box<dyn Error>> {
        Ok(self.hashes.get(hash).cloned())
    }
}

#[test]
fn scan_directory() {
    let tmp_dir = TempDir::new().expect("Could not create tmpdir");
    let tmp_path = tmp_dir.path();

    // Supported file
    let supported_path = tmp_path.join("test.txt");
    let mut supported_file = File::create(supported_path.clone()).expect("");
    supported_file.write_all(b"test\n").expect("");

    // Unsupported file
    let unsupported_path = tmp_path.join("test2.txt");
    let mut unsupported_file = File::create(unsupported_path.clone()).expect("");
    unsupported_file.write_all(b"test2\n").expect("");

    let mut hashes = HashMap::default();
    hashes.insert(
        "f2ca1bb6c7e907d06dafe4687e579fce76b37e4e93b7605022da52e6ccc26fd2".to_owned(),
        VersionedProgramInfo {
            id: "com.example.txt.test".to_owned(),
            title: "test.txt".to_owned(),
            version: Version {
                string: "1.0.0".to_owned(),
                cycle: "1.0".to_owned(),
                major: 1,
                minor: Some(0),
                patch: Some(0),
                extra: None,
            },
        },
    );

    let directories = vec![tmp_dir.path().to_path_buf()];
    let dbs = vec![MockHashDatabase { hashes }];

    let mut results = scan(directories, &dbs).unwrap();

    assert_eq!(results.len(), 2);

    results.sort_by(|lhs, rhs| lhs.file_path.cmp(&rhs.file_path));

    let supported_result = &results[0];
    let unsupported_result = &results[1];

    assert_eq!(supported_result.file_path, supported_path);
    assert_eq!(unsupported_result.file_path, unsupported_path);

    assert_eq!(
        supported_result.file_hash,
        "f2ca1bb6c7e907d06dafe4687e579fce76b37e4e93b7605022da52e6ccc26fd2".to_owned()
    );
    assert_eq!(
        unsupported_result.file_hash,
        "7d6fd7774f0d87624da6dcf16d0d3d104c3191e771fbe2f39c86aed4b2bf1a0f".to_owned()
    );

    assert!(supported_result.program_info.is_some());
    assert!(unsupported_result.program_info.is_none());
}
