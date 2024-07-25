// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use core::str;
use std::{error::Error, process::Command};

use log::error;

use crate::program::{BinaryExtractor, Version};

use super::regex::parse_version;

pub fn info(extractor: BinaryExtractor) -> Result<Vec<Version>, Box<dyn Error>> {
    let r = Command::new(extractor.binary_path)
        .args(extractor.arguments)
        .output();

    match r {
        Ok(output) => {
            let fd = if output.stdout.is_empty() {
                &output.stderr
            } else {
                &output.stdout
            };

            let s = str::from_utf8(fd);

            let r = parse_version(s.unwrap(), &extractor.regex);

            match r {
                Ok(version) => Ok(vec![version]),
                Err(e) => {
                    error!("{e}");
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("{e}");
            Err(Box::new(e))
        }
    }
}
