// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use bollard::{container::ListContainersOptions, secret::ContainerSummary, Docker};
use std::error::Error;

use crate::{
    program::{self, DockerExtractor, Extractor, Version},
    sources::regex,
};

pub struct Connection {
    connection: Docker,
}

impl Connection {
    fn new(connection: Docker) -> Self {
        Connection { connection }
    }

    pub fn connect() -> Result<Connection, Box<dyn Error>> {
        Ok(Connection::new(Docker::connect_with_socket_defaults()?))
    }

    pub async fn info(&self, extractor: &DockerExtractor) -> Result<Vec<Version>, Box<dyn Error>> {
        let result = &self
            .connection
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                //filters,
                ..Default::default()
            }))
            .await?;

        if result.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Did not find matching docker container",
            )));
        }

        let mut versions: Vec<Version> = Vec::default();

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

            if let Ok(Some(v)) = Self::match_oci_version_label(res, &extractor.regex) {
                versions.push(v);
            }
        }

        // Binary matcher
        // @TODO

        Ok(versions)
    }

    fn match_oci_version_label(
        container_summary: &ContainerSummary,
        regex: &str,
    ) -> Result<Option<program::Version>, Box<dyn Error>> {
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

impl Extractor for DockerExtractor {
    async fn version(&self) -> Result<Vec<Version>, Box<dyn Error>> {
        let connection = Connection::connect()?;

        connection.info(self).await
    }

    fn extractor_name() -> &'static str {
        "Docker"
    }
}
