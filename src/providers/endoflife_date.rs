// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::error::Error;

use chrono::NaiveDate;
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

const BASE_URL: &str = "https://endoflife.date/api";

pub async fn get_release_cycles(product: &str) -> Result<Vec<ReleaseCycle>, Box<dyn Error>> {
    let response = reqwest::get(format!("{BASE_URL}/{product}.json")).await?;
    Ok(response.json::<Vec<ReleaseCycle>>().await?)
}

pub async fn get_release_cycle(
    product: &str,
    cycle: CycleId,
) -> Result<ReleaseCycle, Box<dyn Error>> {
    let cycle_text = match cycle {
        CycleId::String(string) => string,
        CycleId::Number(number) => number.to_string(),
    };

    let response = reqwest::get(format!("{BASE_URL}/{product}/{cycle_text}.json")).await?;
    Ok(response.json::<ReleaseCycle>().await?)
}

pub async fn get_all_products() -> Result<Vec<String>, Box<dyn Error>> {
    let response = reqwest::get(format!("{BASE_URL}/all.json")).await?;
    Ok(response.json::<Vec<String>>().await?)
}

#[cfg(test)]
mod tests {

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

    #[tokio::test]
    #[ignore]
    async fn api_get_release_cycles_from_linux() {
        let rcs = get_release_cycles("linux")
            .await
            .expect("Did not receive valid response");
        assert!(!rcs.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn api_get_one_release_cycle_from_linux() {
        let cycle = CycleId::String("6.10".to_string());
        let rcs = get_release_cycle("linux", cycle.clone())
            .await
            .expect("Did not receive valid response");

        assert_eq!(
            rcs.release_date,
            NaiveDate::from_ymd_opt(2024, 7, 14).unwrap()
        );
    }

    #[tokio::test]
    #[ignore]
    async fn api_get_supported_product_list() {
        let products = get_all_products()
            .await
            .expect("Did not receive valid response");

        assert!(!products.is_empty());
    }
}
