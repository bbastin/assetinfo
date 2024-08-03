// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use log::info;
use regex::Regex;

use crate::program::Version;

use super::ExtractorError;

pub fn parse_version(input: &str, regex: &str) -> Result<Version, ExtractorError> {
    info!(r#"Applying "{regex}" to "{input}""#);

    let re = Regex::new(regex)?;
    let matcher = re.captures(input);
    if matcher.is_none() {
        return Err(ExtractorError::VersionError(
            "Regex did not match".to_owned(),
        ));
    }
    let caps = matcher.unwrap();

    let whole_version = caps.name("version").unwrap().as_str().to_string();

    let cycle = caps.name("cycle").unwrap().as_str().to_string();

    let major = caps.name("major").unwrap().as_str().parse::<usize>()?;

    let minor = if let Some(minor) = caps.name("minor") {
        Some(minor.as_str().parse::<usize>()?)
    } else {
        None
    };

    let patch = if let Some(patch) = caps.name("patch") {
        Some(patch.as_str().parse::<usize>()?)
    } else {
        None
    };

    let extra = caps.name("extra").map(|extra| extra.as_str().to_string());

    Ok(Version {
        string: whole_version,
        cycle,
        major,
        minor,
        patch,
        extra,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nginx() {
        let input = "nginx version: nginx/1.18.0 (Ubuntu)";
        let regex = r"^nginx version: nginx/(?<version>(?<cycle>(?<major>\d+)\.(?<minor>\d+))\.(?<patch>\d+))";

        let version = parse_version(input, regex).expect("Could not extract version");

        assert_eq!(version.string, "1.18.0".to_owned());
        assert_eq!(version.cycle, "1.18".to_owned());
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, Some(18));
        assert_eq!(version.patch, Some(0));
        assert_eq!(version.extra, None);
    }

    #[test]
    fn mattermost() {
        let input = "Version: 9.9.2
Build Number: 9962179521
Build Date: Wed Jul 17 13:56:52 UTC 2024
Build Hash: 7bbf7ec130487af9a324040259b2e942d7b9ba3c
Build Enterprise Ready: false";
        let regex = r"^Version: (?<version>(?<cycle>(?<major>\d+)\.(?<minor>\d+))\.(?<patch>\d+))";

        let version = parse_version(input, regex).expect("Could not extract version");

        assert_eq!(version.string, "9.9.2".to_owned());
        assert_eq!(version.cycle, "9.9".to_owned());
        assert_eq!(version.major, 9);
        assert_eq!(version.minor, Some(9));
        assert_eq!(version.patch, Some(2));
        assert_eq!(version.extra, None);
    }
}
