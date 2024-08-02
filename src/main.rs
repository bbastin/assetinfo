// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assetinfo::{
    db::Database,
    program::{Extractor, Program, ProgramInfo, Version},
    providers::endoflife_date::{self, DateOrBool, EndOfLifeDateClient, ReleaseCycle},
};
use chrono::{TimeDelta, Utc};
use clap::{Parser, Subcommand};
use config::Config;
use log::error;
use std::{error::Error, fs, path::PathBuf, process::exit};

mod config;
mod table_view;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Specify path to config file
    #[arg(long, default_value = "./assetinfo-config.json")]
    config_file: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all supported programs
    List {},

    /// Get information for a program
    Info { name: String },

    /// Get information for all supported programs
    InfoAll {},

    /// Update internal database of supported programs
    Update {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config = Config::load(args.config_file)?;

    simple_logger::init_with_level(config.log_level().unwrap_or(log::Level::Warn)).unwrap();

    match args.command {
        Commands::List {} => {
            let db = Database::load(config.database_folder())?;

            table_view::list_supported_programs(&db.supported_programs);
        }
        Commands::Info { name } => {
            let db = Database::load(config.database_folder())?;

            let program = db.get(name.as_str());

            if program.is_none() {
                println!("Could not find any program matching {name}");
                exit(-1);
            }

            let _ = gather_program_info(program.unwrap()).await;
        }
        Commands::InfoAll {} => {
            let db = Database::load(config.database_folder())?;

            // for program in db.supported_programs {
            //     let _ = gather_program_info(program).await;
            // }
            table_view::list_info_all(db.supported_programs).await?;
        }
        Commands::Update {} => {
            update_database(&config).await?;
        }
    }

    Ok(())
}

async fn gather_program_info(program: Program) -> Result<(), Box<dyn Error>> {
    if let Some(binary_extractors) = program.binary {
        for extractor in binary_extractors {
            print_info(extractor, &program.info).await?;
        }
    }

    if let Some(extractor) = program.docker {
        print_info(extractor, &program.info).await?;
    }

    Ok(())
}

async fn print_info<T: Extractor>(
    extractor: T,
    program_info: &ProgramInfo,
) -> Result<(), Box<dyn Error>> {
    if let Some(version) = extractor.version().await? {
        println!(
            "{} ({}) found in Version {}",
            program_info.title,
            T::extractor_name(),
            version.string
        );

        if let Some(ref endoflife_date_id) = program_info.endoflife_date_id {
            const BASE_URL: &str = "https://endoflife.date/api";
            let client = EndOfLifeDateClient::new(BASE_URL);
            if let Ok(cycle_info) = client
                .get_release_cycle(
                    endoflife_date_id,
                    endoflife_date::CycleId::String(version.cycle.clone()),
                )
                .await
            {
                print_end_of_life_info(&version, &cycle_info);
            }
        }
    }
    Ok(())
}

fn print_end_of_life_info(version: &Version, cycle_info: &ReleaseCycle) {
    if let DateOrBool::Date(eol_date) = cycle_info.eol {
        let today = Utc::now().date_naive();

        let remaining_time = eol_date - today;

        if remaining_time > TimeDelta::days(0) {
            println!(
                "Version {} will be supported for {} days ({})",
                version.cycle,
                remaining_time.num_days(),
                eol_date
            );
        } else {
            println!(
                "Version {} is not supported since {} days ({})",
                version.cycle,
                remaining_time.num_days().abs(),
                eol_date
            );
        }
    }
}

async fn update_database(config: &Config) -> Result<(), Box<dyn Error>> {
    let database_folder = config.database_folder();
    if !database_folder.exists() {
        fs::create_dir(database_folder)?;
    }

    if !config.database_folder().is_dir() {
        error!(
            "Database folder path is not a folder: {}",
            database_folder
                .to_str()
                .unwrap_or("<error displaying folder>")
        );
        return Err(Box::new(std::io::Error::from(std::io::ErrorKind::NotFound)));
    }

    let new_db = Database::download_update(config.update_url(), database_folder).await?;

    Database::install_update(&new_db, database_folder).await
}
