// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assetinfo::{
    program::{Extractor, Program, ProgramInfo, Version},
    providers::endoflife_date::{self, Eol, ReleaseCycle},
};
use chrono::{TimeDelta, Utc};
use clap::{Parser, Subcommand};
use db::Database;
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
                println!("{} ({}): ", program.info.title, program.info.id);
            }
        }
        Commands::Info { name } => {
            let p = db.get(name.as_str());

            if p.is_none() {
                println!("Could not find any program matching {name}");
                exit(-1);
            }

            let _ = gather_program_info(p.unwrap()).await;
        }
        Commands::InfoAll {} => {
            for program in db.supported_programs {
                let _ = gather_program_info(program).await;
            }
        }
        Commands::Update {} => {
            unimplemented!("Update is not implemented for now")
        }
    }

    Ok(())
}

async fn gather_program_info(p: Program) -> Result<(), Box<dyn Error>> {
    if let Some(binary_extractors) = p.binary {
        for extractor in binary_extractors {
            print_info(extractor, &p.info).await?
        }
    }

    if let Some(extractor) = p.docker {
        print_info(extractor, &p.info).await?
    }

    Ok(())
}

async fn print_info<T: Extractor>(e: T, p: &ProgramInfo) -> Result<(), Box<dyn Error>> {
    for v in e.version().await? {
        println!(
            "{} ({}) found in Version {}",
            p.title,
            T::extractor_name(),
            v.string
        );

        if let Some(ref endoflife_date_id) = p.endoflife_date_id {
            if let Ok(cycle_info) = endoflife_date::get_release_cycle(
                endoflife_date_id,
                assetinfo::providers::endoflife_date::CycleId::String(v.cycle.clone()),
            )
            .await
            {
                print_end_of_life_info(&v, &cycle_info);
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
