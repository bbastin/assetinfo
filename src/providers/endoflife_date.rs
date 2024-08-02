// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;

use chrono::NaiveDate;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum CycleId {
    String(String),
    Number(i64),
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum Lts {
    Bool(bool),
    String(String),
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum DateOrBool {
    Date(NaiveDate),
    Bool(bool),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseCycle {
    pub cycle: Option<CycleId>,
    pub release_date: NaiveDate,
    pub eol: DateOrBool,
    pub latest: String,
    pub link: Option<String>,
    pub lts: Lts,
    pub support: Option<DateOrBool>,
    pub discontinued: Option<DateOrBool>,
}

pub struct EndOfLifeDateClient {
    base_url: String,
}

impl EndOfLifeDateClient {
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        EndOfLifeDateClient {
            base_url: base_url.to_owned(),
        }
    }

    pub async fn get_release_cycles(
        &self,
        product: &str,
    ) -> Result<Vec<ReleaseCycle>, Box<dyn Error>> {
        let url = format!("{}/{product}.json", self.base_url);
        info!("Retrieving ReleaseCycles for {product} from {url}");

        let response = reqwest::get(url).await?;
        info!("Recieved response {}", response.status());

        Ok(response.json::<Vec<ReleaseCycle>>().await?)
    }

    pub async fn get_release_cycle(
        &self,
        product: &str,
        cycle: CycleId,
    ) -> Result<ReleaseCycle, Box<dyn Error>> {
        let cycle_text = match cycle {
            CycleId::String(string) => string,
            CycleId::Number(number) => number.to_string(),
        };
        let url = format!("{}/{product}/{cycle_text}.json", self.base_url);
        info!("Retrieving ReleaseCycle {cycle_text} for {product} from {url}");

        let response = reqwest::get(url).await?;
        info!("Recieved response {}", response.status());

        Ok(response.json::<ReleaseCycle>().await?)
    }

    pub async fn get_all_products(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let url = format!("{}/all.json", self.base_url);
        info!("Retrieving supported products from {url}");

        let response = reqwest::get(url).await?;
        info!("Recieved response {}", response.status());

        Ok(response.json::<Vec<String>>().await?)
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    #[test]
    fn deserialize_release_cycle() {
        let test = r#"{
            "cycle": "21.04",
            "lts": false,
            "releaseDate": "2021-04-22",
            "support": "2022-01-01",
            "eol": "2022-01-01",
            "latest": "21.04",
            "link": "https://wiki.ubuntu.com/HirsuteHippo/ReleaseNotes/"
        }"#;

        let rc: ReleaseCycle = serde_json::from_str(test).expect("Deserialization failed");

        assert_eq!(rc.cycle, Some(CycleId::String("21.04".to_string())));
        assert_eq!(rc.lts, Lts::Bool(false));
        assert_eq!(
            rc.release_date,
            NaiveDate::from_ymd_opt(2021, 4, 22).unwrap()
        );
        assert_eq!(
            rc.support,
            Some(DateOrBool::Date(
                NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
            ))
        );
        assert_eq!(
            rc.eol,
            DateOrBool::Date(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap())
        );
        assert_eq!(rc.latest, "21.04");
        assert_eq!(
            rc.link,
            Some("https://wiki.ubuntu.com/HirsuteHippo/ReleaseNotes/".to_string())
        );
    }

    #[test(tokio::test)]
    async fn api_get_release_cycles_from_linux() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Create a mock
        let mock = server
            .mock("GET", "/linux.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
{
    "cycle": "6.10",
    "releaseDate": "2024-07-14",
    "eol": "2024-11-14",
    "latest": "6.10.2",
    "latestReleaseDate": "2024-07-27",
    "lts": false
}]"#,
            )
            .create_async()
            .await;

        let client = EndOfLifeDateClient::new(&url);

        let rcs = client
            .get_release_cycles("linux")
            .await
            .expect("Did not receive valid response");
        assert!(!rcs.is_empty());

        mock.assert_async().await;
    }

    #[test(tokio::test)]
    async fn api_get_one_release_cycle_from_linux() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Create a mock
        let mock = server
            .mock("GET", "/linux/6.10.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
    "releaseDate": "2024-07-14",
    "eol": "2024-11-14",
    "latest": "6.10.2",
    "latestReleaseDate": "2024-07-27",
    "lts": false
}"#,
            )
            .create_async()
            .await;

        let client = EndOfLifeDateClient::new(&url);

        let cycle = CycleId::String("6.10".to_string());
        let rcs = client
            .get_release_cycle("linux", cycle.clone())
            .await
            .expect("Did not receive valid response");

        assert_eq!(
            rcs.release_date,
            NaiveDate::from_ymd_opt(2024, 7, 14).unwrap()
        );

        mock.assert_async().await;
    }

    #[test(tokio::test)]
    async fn api_get_supported_product_list() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        // Create a mock
        let mock = server
            .mock("GET", "/all.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"["linux"]"#)
            .create_async()
            .await;

        let client = EndOfLifeDateClient::new(&url);

        let products = client
            .get_all_products()
            .await
            .expect("Did not receive valid response");

        assert!(!products.is_empty());

        mock.assert_async().await;
    }
}
