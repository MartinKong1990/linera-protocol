// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use linera_service::storage::StorageConfig;
use linera_views::common::CommonStoreConfig;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Clear database", about = "A tool for cleaning up a database")]
struct DatabaseToolOptions {
    /// Subcommands. Acceptable values are run and generate.
    #[structopt(subcommand)]
    command: DatabaseToolCommand,
}

#[derive(StructOpt)]
enum DatabaseToolCommand {
    /// Subcommands. Acceptable values are delete_all, delete_single, initialize

    /// Delete all the entries of the database
    #[structopt(name = "delete_all")]
    DeleteAll {
        /// Storage configuration for the blockchain history.
        #[structopt(long = "storage")]
        storage_config: String,
    },

    /// Delete a single table from the database
    #[structopt(name = "delete_single")]
    DeleteSingle {
        /// Storage configuration for the blockchain history.
        #[structopt(long = "storage")]
        storage_config: String,
    },

    /// Check existence of a database
    #[structopt(name = "check_existence")]
    CheckExistence {
        /// Storage configuration for the blockchain history.
        #[structopt(long = "storage")]
        storage_config: String,
    },

    /// Check absence of a database
    #[structopt(name = "check_absence")]
    CheckAbsence {
        /// Storage configuration for the blockchain history.
        #[structopt(long = "storage")]
        storage_config: String,
    },

    /// Initialize a table in the database
    #[structopt(name = "initialize")]
    Initialize {
        /// Storage configuration for the blockchain history.
        #[structopt(long = "storage")]
        storage_config: String,
    },

    /// List the tables of the database
    #[structopt(name = "list_tables")]
    ListTables {
        /// Storage configuration for the blockchain history.
        #[structopt(long = "storage")]
        storage_config: String,
    },
}

async fn evaluate_options(options: DatabaseToolOptions) -> Result<i32, anyhow::Error> {
    let common_config = CommonStoreConfig::default();
    match options.command {
        DatabaseToolCommand::DeleteAll { storage_config } => {
            let storage_config: StorageConfig = storage_config.parse()?;
            let full_storage_config = storage_config.add_common_config(common_config).await?;
            full_storage_config.delete_all().await?;
        }
        DatabaseToolCommand::DeleteSingle { storage_config } => {
            let storage_config: StorageConfig = storage_config.parse()?;
            let full_storage_config = storage_config.add_common_config(common_config).await?;
            full_storage_config.delete_single().await?;
        }
        DatabaseToolCommand::CheckExistence { storage_config } => {
            let storage_config: StorageConfig = storage_config.parse()?;
            let full_storage_config = storage_config.add_common_config(common_config).await?;
            let test = full_storage_config.test_existence().await?;
            if test {
                tracing::info!("The database does exist");
                return Ok(0);
            } else {
                tracing::info!("The database does not exist");
                return Ok(1);
            }
        }
        DatabaseToolCommand::CheckAbsence { storage_config } => {
            let storage_config: StorageConfig = storage_config.parse()?;
            let full_storage_config = storage_config.add_common_config(common_config).await?;
            let test = full_storage_config.test_existence().await?;
            if test {
                tracing::info!("The database does exist");
                return Ok(1);
            } else {
                tracing::info!("The database does not exist");
                return Ok(0);
            }
        }
        DatabaseToolCommand::Initialize { storage_config } => {
            let storage_config: StorageConfig = storage_config.parse()?;
            let full_storage_config = storage_config.add_common_config(common_config).await?;
            full_storage_config.initialize().await?;
        }
        DatabaseToolCommand::ListTables { storage_config } => {
            let storage_config: StorageConfig = storage_config.parse()?;
            let full_storage_config = storage_config.add_common_config(common_config).await?;
            let tables = full_storage_config.list_tables().await?;
            println!("The list of tables is {:?}", tables);
        }
    }
    tracing::info!("Successful execution of linera-db");
    Ok(0)
}

#[tokio::main]
async fn main() {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .init();

    let options = DatabaseToolOptions::from_args();
    let error_code = match evaluate_options(options).await {
        Ok(code) => code,
        Err(msg) => {
            tracing::error!("Error is {:?}", msg);
            2
        }
    };
    process::exit(error_code);
}
