// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::extractor::{binary::BinaryExtractor, docker::DockerExtractor};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn version(major: usize, minor: Option<usize>, patch: Option<usize>) -> Version {
        Version {
            major,
            minor,
            patch,
            string: String::default(),
            cycle: String::default(),
            extra: None,
        }
    }

    #[test]
    fn comparisons() {
        let v111 = version(1, Some(1), Some(1));
        let v112 = version(1, Some(1), Some(2));
        let v121 = version(1, Some(2), Some(1));
        let v122 = version(1, Some(2), Some(2));
        assert!(v111 < v112);
        assert!(v111 < v121);
        assert!(v111 < v112);
        assert!(v111 < v122);

        assert!(v112 < v121);
        assert!(v112 < v122);
        assert!(v121 < v122);

        assert_eq!(v111, v111);
        assert_eq!(v112, v112);
        assert_eq!(v121, v121);
        assert_eq!(v122, v122);

        assert_ne!(v111, v112);
        assert_ne!(v111, v121);
        assert_ne!(v111, v121);
        assert_ne!(v111, v122);

        assert!(v111 > version(1, None, None));
        assert!(v111 > version(1, Some(1), None));
        assert!(v111 < version(2, Some(1), None));
        assert!(v111 < version(2, None, None));
    }
}
