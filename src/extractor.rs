// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;

use crate::program::Version;

pub mod binary;
pub mod docker;
pub mod regex;

pub trait Extractor {
    #[allow(async_fn_in_trait)]
    async fn version(&self) -> Result<Option<Version>, Box<dyn Error>>;

    fn extractor_name() -> &'static str;
}
