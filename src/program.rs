// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};

pub trait Extractor {
    #[allow(async_fn_in_trait)]
    async fn version(&self) -> Result<Option<Version>, Box<dyn Error>>;

    fn extractor_name() -> &'static str;
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct BinaryExtractor {
    pub path: PathBuf,
    pub user: Option<String>,
    pub arguments: Vec<String>,
    pub regex: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct DockerExtractor {
    pub image_name: String,
    pub binary_path: Option<PathBuf>,
    pub arguments: Option<Vec<String>>,
    pub regex: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct ProgramInfo {
    pub id: String,
    pub title: String,
    pub endoflife_date_id: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Program {
    pub info: ProgramInfo,
    pub binary: Option<Vec<BinaryExtractor>>,
    pub docker: Option<DockerExtractor>,
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
