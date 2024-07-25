// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use bollard::{container::ListContainersOptions, Docker};
use log::warn;
use std::error::Error;

use crate::program::{Program, Version};

use super::regex::parse_version;

pub struct Connection {
    connection: Docker,
}

impl Connection {
    fn new(connection: Docker) -> Self {
        Connection { connection }
    }

    pub async fn info(&self, program: Program) -> Result<Vec<Version>, Box<dyn Error>> {
        let extractor = program.docker.unwrap();

        // let mut filters = HashMap::new();
        // filters.insert("ancestor".to_string(), vec![extractor.image_name]);

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

            if let Some(labels) = res.labels.clone() {
                const VERSION_LABEL: &str = "org.opencontainers.image.version";

                if let Some(version_string) = labels.get(VERSION_LABEL) {
                    match parse_version(version_string, &extractor.regex) {
                        Ok(v) => versions.push(v),
                        Err(e) => warn!("{e}"),
                    }
                }
            }
        }

        // Binary matcher
        // @TODO

        Ok(versions)
    }
}

pub fn connect() -> Result<Connection, Box<dyn Error>> {
    Ok(Connection::new(Docker::connect_with_socket_defaults()?))
}
