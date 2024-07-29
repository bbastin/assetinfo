// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use core::str;
use std::{
    error::Error,
    io::Error as IoError,
    process::{Command, Output},
};

use log::{error, info};

use crate::program::{BinaryExtractor, Extractor, Version};

use super::regex::parse_version;

impl Extractor for BinaryExtractor {
    async fn version(&self) -> Result<Vec<Version>, Box<dyn Error>> {
        if !self.path.exists() {
            return Ok(Vec::default());
        }

        let r = if self.user.is_some() {
            self.run_as_other_user_sudo()
        } else {
            self.run_as_user()
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

                if !output.status.success() {
                    return Err(Box::new(IoError::new(
                        std::io::ErrorKind::Other,
                        s.unwrap(),
                    )));
                }

                let r = parse_version(s.unwrap(), &self.regex);

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

    fn extractor_name() -> &'static str {
        "Binary"
    }
}

impl BinaryExtractor {
    fn run_as_user(&self) -> std::io::Result<Output> {
        Command::new(self.path.clone())
            .args(self.arguments.clone())
            .output()
    }

    #[allow(dead_code)]
    fn run_as_other_user_systemd(&self) -> std::io::Result<Output> {
        let user = self.user.clone().unwrap();

        let mut args: Vec<String> = vec![
            "--pty".to_string(),
            //"--wait".to_string(),
            //"--collect".to_string(),
            //"--service-type=exec".to_string(),
            "--quiet".to_string(),
            format!("--uid={user}",),
            self.path.to_str().unwrap().to_string(),
        ];

        for a in self.arguments.clone() {
            args.push(a);
        }

        info!("Running /usr/bin/systemd-run {args:?}",);

        Command::new("/usr/bin/systemd-run").args(args).output()
    }

    fn run_as_other_user_sudo(&self) -> std::io::Result<Output> {
        let user = self.user.clone().unwrap();

        let mut args: Vec<String> = vec![
            "-u".to_string(),
            user,
            self.path.to_str().unwrap().to_string(),
        ];

        for a in self.arguments.clone() {
            args.push(a);
        }

        info!("Running /usr/bin/sudo {args:?}",);

        Command::new("/usr/bin/sudo").args(args).output()
    }
}

#[cfg(test)]
mod tests {

    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test() {
        let tmp_dir = TempDir::new().expect("Could not create tmpdir");
        let file_path = tmp_dir.path().join("testprogram");
        let mut tmp_file = File::create(file_path.clone()).expect("Could not create tmpfile");
        writeln!(tmp_file, "#!/bin/sh\necho 1.2.3").expect("Could not write to tmpfile");
        tmp_file.flush().expect("Could not flush tmpfile");
        drop(tmp_file);
        Command::new("/bin/chmod")
            .arg("+x")
            .arg(file_path.clone())
            .output()
            .expect("Could not set permission on tmpfile");

        let extractor = BinaryExtractor {
            path: file_path.clone(),
            user: None,
            arguments: Vec::default(),
            regex: "^(?<version>(?<cycle>(?<major>\\d+))\\.(?<minor>\\d+)\\.(?<patch>\\d+))"
                .to_string(),
        };

        let res = extractor.version().await;
        if let Err(e) = res {
            panic!("{e}");
        }
        assert!(res.is_ok());

        let version = res.unwrap();
        assert_eq!(version.len(), 1);

        let version = &version[0];
        assert_eq!(version.string, "1.2.3");

        fs::remove_file(file_path).expect("Could not delete tmpfile");
    }
}
