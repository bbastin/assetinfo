// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use thiserror::Error;

use crate::program::Version;

pub mod binary;
pub mod docker;
pub mod regex;

pub trait Extractor {
    #[allow(async_fn_in_trait)]
    async fn version(&self) -> Result<Option<Version>, ExtractorError>;

    fn extractor_name() -> &'static str;
}

#[derive(Error, Debug)]
pub enum ExtractorError {
    #[error("Could not find valid Version. Reason: {0}")]
    VersionError(String),

    #[error("Could not construct regex. Reason: {0}")]
    RegexError(#[from] ::regex::Error),

    #[error("Could not parse Version number: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error on Docker Connection: {0}")]
    DockerError(#[from] bollard::errors::Error),
}
