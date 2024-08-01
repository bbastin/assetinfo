// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assetinfo::{
    program::{Extractor, Program, ProgramInfo, Version},
    providers::endoflife_date::{self, CycleId, DateOrBool, Eol, ReleaseCycle},
};
use chrono::Utc;
use std::error::Error;
use tabled::{
    settings::{object::Rows, themes::Colorization, Color, Panel, Style},
    Table, Tabled,
};

pub(crate) fn list_supported_programs(programs: &[Program]) {
    #[derive(Tabled)]
    struct ProgramDisplayInfo {
        #[tabled(rename = "Program Name")]
        title: String,
        #[tabled(rename = "Program ID")]
        id: String,
        #[tabled(rename = "Binary")]
        binary: bool,
        #[tabled(rename = "Docker")]
        docker: bool,
    }

    let mut data = programs
        .iter()
        .map(|p| ProgramDisplayInfo {
            title: p.info.title.clone(),
            id: p.info.id.clone(),
            binary: p.binary.as_ref().is_some_and(|b| !b.is_empty()),
            docker: p.docker.is_some(),
        })
        .collect::<Vec<_>>();

    data.sort_by(|a, b| a.title.partial_cmp(&b.title).unwrap());

    let mut table = Table::new(data);
    table
        .with(Style::psql())
        .with(Panel::header("Supported programs"));

    println!("{table}");
}

#[derive(Tabled, Clone)]
struct ProgramDisplayVersion {
    #[tabled(rename = "Program Name")]
    title: String,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Release Cycle")]
    cycle: String,
    #[tabled(rename = "Updates until")]
    updates_until: String,
    #[tabled(rename = "Security Updates until")]
    security_until: String,
}

fn version_row(
    p: &ProgramInfo,
    v: &Version,
    r: &Option<ReleaseCycle>,
    source: &str,
) -> ProgramDisplayVersion {
    let today = Utc::now().date_naive();

    let security_until = match r {
        Some(r) => {
            if let Eol::Date(eol_date) = r.eol {
                let remaining_time = eol_date - today;

                format!("{} ({} days)", eol_date, remaining_time.num_days(),)
            } else {
                "Unknown".to_string()
            }
        }
        None => "Unknown".to_string(),
    };

    let updates_until = match r {
        Some(r) => match r.support {
            Some(DateOrBool::Date(date)) => {
                let remaining_time = date - today;
                format!("{} ({} days)", date, remaining_time.num_days(),)
            }
            Some(DateOrBool::Bool(supported)) => {
                format!("{supported}")
            }
            None => security_until.clone(),
        },
        None => security_until.clone(),
    };

    let cycle = if let Some(r) = r {
        format!("{} ({})", v.cycle, r.latest)
    } else {
        v.cycle.clone()
    };

    ProgramDisplayVersion {
        title: p.title.clone(),
        source: source.to_string(),
        version: v.string.clone(),
        cycle,
        updates_until,
        security_until,
    }
}

#[derive(Clone, Copy)]
enum SupportState {
    Supported,
    Security,
    #[allow(dead_code)]
    AlmostEol,
    Unsupported,
    Unknown,
}

async fn get_release_cycle(p: &ProgramInfo, v: &Version) -> Option<ReleaseCycle> {
    match p.endoflife_date_id {
        Some(ref id) => {
            match endoflife_date::get_release_cycle(id, CycleId::String(v.cycle.clone())).await {
                Ok(e) => Some(e),
                Err(_e) => None,
            }
        }
        None => None,
    }
}

fn get_display_release_cycle(release_cycle: &Option<ReleaseCycle>) -> SupportState {
    if let Some(release_cycle) = release_cycle {
        let today = Utc::now().date_naive();

        if let Eol::Date(eol) = release_cycle.eol {
            if eol < today {
                SupportState::Unsupported
            } else {
                match release_cycle.support {
                    Some(DateOrBool::Date(supported_until)) => {
                        if supported_until < today {
                            SupportState::Security
                        } else {
                            SupportState::Supported
                        }
                    }
                    Some(DateOrBool::Bool(is_supported)) => {
                        if is_supported {
                            SupportState::Supported
                        } else {
                            SupportState::Security
                        }
                    }
                    None => SupportState::Supported,
                }
            }
        } else {
            SupportState::Supported
        }
    } else {
        SupportState::Unknown
    }
}

async fn run_extractor<T: Extractor>(
    p: &ProgramInfo,
    extractor: &T,
) -> Option<(ProgramDisplayVersion, SupportState)> {
    let v = extractor.version().await;
    if let Ok(Some(v)) = v {
        let release_cycle = get_release_cycle(p, &v).await;
        let row = version_row(p, &v, &release_cycle, "Binary");
        return Some((row, get_display_release_cycle(&release_cycle)));
    }
    None
}

pub(crate) async fn list_info_all(programs: Vec<Program>) -> Result<(), Box<dyn Error>> {
    let mut rows: Vec<(ProgramDisplayVersion, SupportState)> = Vec::default();

    let default = Color::FG_BRIGHT_BLACK;
    let supported = Color::FG_GREEN;
    let security = Color::FG_BLUE;
    let warn = Color::FG_YELLOW;
    let unsupported = Color::BOLD | Color::FG_RED;

    for p in programs {
        // Binary
        if let Some(binary_extractors) = p.binary {
            for extractor in binary_extractors {
                if let Some(row) = run_extractor(&p.info, &extractor).await {
                    rows.push(row);
                }
            }
        }

        // Docker
        if let Some(extractor) = p.docker {
            if let Some(row) = run_extractor(&p.info, &extractor).await {
                rows.push(row);
            }
        }
    }

    let mut table = Table::new(rows.iter().map(|r| r.0.clone()));
    table
        .with(Style::psql())
        .with(Panel::header("Detected programs"));

    let support_states: Vec<_> = rows.iter().map(|r| r.1).collect();

    for (i, state) in support_states.iter().enumerate() {
        let color = match state {
            SupportState::Supported => supported.clone(),
            SupportState::Security => security.clone(),
            SupportState::AlmostEol => warn.clone(),
            SupportState::Unsupported => unsupported.clone(),
            SupportState::Unknown => default.clone(),
        };

        table.with(Colorization::exact([color], Rows::single(i + 2)));
    }

    println!("{table}");

    Ok(())
}
