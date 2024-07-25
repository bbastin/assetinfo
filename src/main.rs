// SPDX-FileCopyrightText: 2024 Benedikt Bastin
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Parser, Subcommand};
use db::Database;
use eolmon::sources::binary;
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
    Info { name: String },

    /// Update internal database of supported programs
    Update {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init().unwrap();

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

            let p = p.unwrap();

            // let docker_connection = docker::connect()?;

            // let info = docker_connection.info(p).await.unwrap();

            let info = binary::info(p.binary.unwrap())?;
            for v in info {
                println!("{:?}", v);
            }
        }
        Commands::Update {} => {
            unimplemented!("Update is not implemented for now")
        }
    }

    Ok(())
}
