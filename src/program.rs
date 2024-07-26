// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone)]
pub struct BinaryExtractor {
    pub path: PathBuf,
    pub user: Option<String>,
    pub arguments: Vec<String>,
    pub regex: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DockerExtractor {
    pub image_name: String,
    pub binary_path: PathBuf,
    pub arguments: Vec<String>,
    pub regex: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Program {
    pub id: String,
    pub title: String,
    pub binary: Option<Vec<BinaryExtractor>>,
    pub docker: Option<DockerExtractor>,
    pub endoflife_date_id: Option<String>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Version {
    pub string: String,
    pub cycle: String,
    pub major: usize,
    pub minor: Option<usize>,
    pub patch: Option<usize>,
    pub extra: Option<String>,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.major.partial_cmp(&other.major) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.minor.partial_cmp(&other.minor) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.patch.partial_cmp(&other.patch)
    }
}
