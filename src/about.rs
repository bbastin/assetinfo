// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;

use async_compression::tokio::bufread::ZstdDecoder;
use tokio::io::AsyncReadExt;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

const THIRDPARTY_LICENSES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/THIRDPARTY.toml.zst"));

async fn extract_thirdparty_licenses() -> Result<String, Box<dyn Error>> {
    let mut reader = ZstdDecoder::new(tokio::io::BufReader::new(THIRDPARTY_LICENSES));
    let mut data: Vec<u8> = vec![];
    reader.read_to_end(&mut data).await?;

    Ok(String::from_utf8(data)?)
}

async fn display_thirdparty_licenses() {
    let license_file = extract_thirdparty_licenses().await.unwrap_or_else(|_| panic!("THIRDPARTY license file could not be decoded. Visit {REPOSITORY} for more information."));

    println!("{license_file}");
}

pub async fn about(thirdparty: bool) {
    println!("{NAME} {VERSION} (C) {AUTHORS} {LICENSE}");
    println!("Visit {HOMEPAGE} or {REPOSITORY} for more information.");

    if thirdparty {
        display_thirdparty_licenses().await;
    } else {
        println!("Run again with --thirdparty or visit {REPOSITORY} to see information on thirdparty dependencies.");
    }
}
