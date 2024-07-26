// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::{TimeDelta, Utc};
use clap::{Parser, Subcommand};
use db::Database;
use eolmon::{
    program::{Program, Version},
    providers::endoflife_date::{self, Eol, ReleaseCycle},
    sources::binary,
};
use log::info;
use std::{env, error::Error, process::exit};

mod db;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all supported programs
    List {},

    /// Get information for a program
    Info {
        name: String,
    },

    InfoAll {},

    /// Update internal database of supported programs
    Update {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Warn).unwrap();

    let args = Args::parse();

    let db_path = env::current_dir()?.join("db");

    let db = Database::load(db_path)?;

    info!("Loaded database {}", db.path.display());

    match args.command {
        Commands::List {} => {
            println!("Supported programs:");
            for program in db.supported_programs {
                println!("{} ({}): ", program.title, program.id);
            }
        }
        Commands::Info { name } => {
            let p = db.get(name.as_str());

            if p.is_none() {
                println!("Could not find any program matching {name}");
                exit(-1);
            }

            let _ = print_info(p.unwrap()).await;
        }
        Commands::InfoAll {} => {
            for program in db.supported_programs {
                let _ = print_info(program).await;
            }
        }
        Commands::Update {} => {
            unimplemented!("Update is not implemented for now")
        }
    }

    Ok(())
}

async fn print_info(p: Program) -> Result<(), Box<dyn Error>> {
    if let Some(binary_extractors) = p.binary {
        for extractor in binary_extractors {
            for v in binary::info(&extractor)? {
                println!("{} found in Version {}", p.title, v.string);

                if let Some(ref endoflife_date_id) = p.endoflife_date_id {
                    if let Ok(cycle_info) = endoflife_date::get_release_cycle(
                        endoflife_date_id,
                        eolmon::providers::endoflife_date::CycleId::String(v.cycle.clone()),
                    )
                    .await
                    {
                        print_end_of_life_info(&v, &cycle_info);
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_end_of_life_info(v: &Version, cycle_info: &ReleaseCycle) {
    if let Eol::Date(eol_date) = cycle_info.eol {
        let today = Utc::now().date_naive();

        let remaining_time = eol_date - today;

        if remaining_time > TimeDelta::days(0) {
            println!(
                "Version {} will be supported for {} days ({})",
                v.cycle,
                remaining_time.num_days(),
                eol_date
            );
        } else {
            println!(
                "Version {} is not supported since {} days ({})",
                v.cycle,
                remaining_time.num_days().abs(),
                eol_date
            );
        }
    }
}
