use clap::Parser;
use serde::Deserialize;

/// A simple typescript definition generator for SurrealDB
#[derive(Parser, Debug, Deserialize)]
#[command(author, version, about, long_about = None)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The address to the SurrealDB instance
    #[arg(short, long, default_value_t = default_address())]
    #[serde(default = "default_address")]
    pub address: String,

    /// The root username for the SurrealDB instance
    #[arg(short, long, default_value_t = default_username())]
    #[serde(default = "default_username")]
    pub username: String,

    /// The root password for the SurrealDB instance
    #[arg(short, long, default_value_t = default_password())]
    #[serde(default = "default_password")]
    pub password: String,

    /// The namespace to use
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// The database to use
    #[arg(short, long)]
    pub database: Option<String>,

    /// Treat record types as FETCHED version of the linked table
    #[arg(short, long)]
    #[serde(default)]
    pub links_fetched: bool,

    /// Use the utility types from the JS SDK in the output file
    #[arg(short, long, default_value_t = default_target_sdk())]
    #[serde(default = "default_target_sdk")]
    pub target_sdk: bool,

    /// Store generated table and field metadata into the database
    #[arg(short, long)]
    #[serde(default)]
    pub store_meta_in_db: bool,

    /// Name of the table to use when the 'store-in-db' flag is enabled
    #[arg(short, long, default_value_t = default_metadata_table())]
    #[serde(default = "default_metadata_table")]
    pub metadata_table_name: String,

    /// Skip adding the table meta descriptors to the output ts file
    #[arg(long)]
    #[serde(default)]
    pub no_meta: bool,

    /// Skip the generation of the typescript definition file
    #[arg(long)]
    #[serde(default)]
    pub skip_ts_generation: bool,

    /// Path where the typescript defintion file will be generated
    #[arg(short, long, default_value_t = default_output())]
    #[serde(default = "default_output")]
    pub output: String,

    /// Path to the configuration JSON file
    #[arg(short, long)]
    pub config_file_path: Option<String>,
}

fn default_address() -> String {
    "http://localhost:8000".to_string()
}

fn default_username() -> String {
    "root".to_string()
}

fn default_password() -> String {
    "root".to_string()
}

fn default_target_sdk() -> bool {
    true
}

fn default_metadata_table() -> String {
    "table_meta".to_string()
}

fn default_output() -> String {
    "db.ts".to_string()
}

use std::fs::File;
use std::io::BufReader;

pub(crate) fn get_config() -> anyhow::Result<Config> {
    let config = Config::parse();

    return if let Some(path) = &config.config_file_path {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    } else {
        Ok(config)
    };
}
