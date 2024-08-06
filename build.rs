// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{env, fs::File, io::Write, path::Path};

fn main() {
    println!("cargo::rerun-if-changed=THIRDPARTY.toml");

    let thirdparty_licenses = include_str!("THIRDPARTY.toml").as_bytes();

    let enc = zstd::encode_all(thirdparty_licenses, 19).expect("Zstd encoding failed");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("THIRDPARTY.toml.zst");

    let mut output = File::create(path).expect("Could not create zstd file");
    output
        .write_all(&enc)
        .expect("Could not write to zstd file");
}
