// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use bollard::{container::ListContainersOptions, secret::ContainerSummary, Docker};
use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};

use crate::{extractor::regex, program::Version};

use super::{Extractor, ExtractorError};

pub struct Connection {
    connection: Docker,
}

impl Connection {
    fn new(connection: Docker) -> Self {
        Connection { connection }
    }

    pub fn connect() -> Result<Connection, bollard::errors::Error> {
        Ok(Connection::new(Docker::connect_with_socket_defaults()?))
    }

    pub async fn info(
        &self,
        extractor: &DockerExtractor,
    ) -> Result<Option<Version>, ExtractorError> {
        let result = &self
            .connection
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                //filters,
                ..Default::default()
            }))
            .await?;

        // Label matcher
        for res in result {
            if res.image.is_none()
                || !res
                    .image
                    .clone()
                    .unwrap()
                    .starts_with(extractor.image_name.as_str())
            {
                continue;
            }

            if let Ok(Some(version)) = Self::match_oci_version_label(res, &extractor.regex) {
                return Ok(Some(version));
            }
        }

        // Binary matcher
        // @TODO

        Ok(None)
    }

    fn match_oci_version_label(
        container_summary: &ContainerSummary,
        regex: &str,
    ) -> Result<Option<Version>, Box<dyn Error>> {
        if let Some(labels) = container_summary.labels.clone() {
            const VERSION_LABEL: &str = "org.opencontainers.image.version";

            let label = labels.get(VERSION_LABEL);

            if let Some(str) = label {
                let version = regex::parse_version(str, regex)?;

                return Ok(Some(version));
            }
        }
        Ok(None)
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub struct DockerExtractor {
    pub image_name: String,
    pub binary_path: Option<PathBuf>,
    pub arguments: Option<Vec<String>>,
    pub regex: String,
}

impl Extractor for DockerExtractor {
    async fn version(&self) -> Result<Option<Version>, ExtractorError> {
        let connection = Connection::connect()?;

        connection.info(self).await
    }

    fn extractor_name() -> &'static str {
        "Docker"
    }
}
