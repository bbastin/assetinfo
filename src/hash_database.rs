// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;

use crate::program::Version;

#[derive(Clone, Debug)]
pub struct VersionedProgramInfo {
    pub id: String,
    pub title: String,
    pub version: Version,
}

pub trait HashDatabase {
    fn get(&self, hash: &str) -> Result<Option<VersionedProgramInfo>, Box<dyn Error>>;
}
