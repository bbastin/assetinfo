// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use core::str;
use std::{
    error::Error,
    process::{Command, Output},
};

use log::{error, info};

use crate::program::{BinaryExtractor, Version};

use super::regex::parse_version;

pub fn info(extractor: &BinaryExtractor) -> Result<Vec<Version>, Box<dyn Error>> {
    let r = if extractor.user.is_some() {
        run_as_other_user_sudo(extractor)
    } else {
        run_as_user(extractor)
    };

    info!("Command executed");

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

pub fn run_as_user(extractor: &BinaryExtractor) -> std::io::Result<Output> {
    Command::new(extractor.binary_path.clone())
        .args(extractor.arguments.clone())
        .output()
}

pub fn run_as_other_user_systemd(extractor: &BinaryExtractor) -> std::io::Result<Output> {
    let user = extractor.user.clone().unwrap();

    let mut args: Vec<String> = vec![
        "--pty".to_string(),
        //"--wait".to_string(),
        //"--collect".to_string(),
        //"--service-type=exec".to_string(),
        "--quiet".to_string(),
        format!("--uid={user}",),
        extractor.binary_path.to_str().unwrap().to_string(),
    ];

    for a in extractor.arguments.clone() {
        args.push(a);
    }

    info!("Running /usr/bin/systemd-run {args:?}",);

    Command::new("/usr/bin/systemd-run").args(args).output()
}

pub fn run_as_other_user_sudo(extractor: &BinaryExtractor) -> std::io::Result<Output> {
    let user = extractor.user.clone().unwrap();

    let mut args: Vec<String> = vec![
        "-u".to_string(),
        user,
        extractor.binary_path.to_str().unwrap().to_string(),
    ];

    for a in extractor.arguments.clone() {
        args.push(a);
    }

    info!("Running /usr/bin/sudo {args:?}",);

    Command::new("/usr/bin/sudo").args(args).output()
}
