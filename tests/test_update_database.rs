// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assetinfo::db::Database;
use tempfile::TempDir;

#[tokio::test]
async fn download_from_production() {
    let tmp_dir = TempDir::new().expect("Could not create tmpdir");

    let update_file = Database::download_update("https://db.assetinfo.de/d45ab56217ea96762255f6f8840c4625ed5a025760169038f5aa2454c109cd26.tar.zstd", tmp_dir.path()).await.expect("Download failed");

    Database::install_update(&update_file, tmp_dir.path())
        .await
        .expect("Installation failed");
}
