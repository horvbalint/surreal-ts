// A simple to use typescript type definition generator for SurrealDB
// Copyright (C) 2023  Horváth Bálint

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see https://www.gnu.org/licenses/.

use clap::Parser;
use surreal_ts_core::Generator;
use surrealdb::opt::auth::Root;

mod meta;
mod ts;
/// A simple typescript definition generator for SurrealDB
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// The connection url to the SurrealDB instance
    #[arg(short, long, default_value = "http://localhost:8000")]
    connection_url: String,

    /// The root username for the SurrealDB instance
    #[arg(short, long, default_value = "root")]
    username: String,

    /// The root password for the SurrealDB instance
    #[arg(short, long, default_value = "root")]
    password: String,

    /// The namespace to use
    #[arg(short, long)]
    namespace: String,

    /// The database to use
    #[arg(short, long)]
    database: String,

    /// Store generated table and field metadata into the database
    #[arg(short, long)]
    store_in_db: bool,

    /// Name of the table to use when the 'store-in-db' flag is enabled
    #[arg(short, long, default_value = "table_meta")]
    metadata_table_name: String,

    /// Skip the generation of a typescript definition file
    #[arg(long)]
    skip_ts_generation: bool,

    /// The path where the typescript defintion file will be generated
    #[arg(short, long, default_value = "db.d.ts")]
    output: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    // let mut db = connect_to_db(&args.connection_url).await?;
    let mut db = surrealdb::engine::any::connect(&args.connection_url).await?;

    db.signin(Root {
        username: &args.username,
        password: &args.password,
    })
    .await?;
    db.use_ns(&args.namespace).use_db(&args.database).await?;

    let mut tables = Generator::process(&mut db)
        .await?
        .into_iter()
        .filter(|(name, _)| name != &args.metadata_table_name)
        .collect();

    println!();

    if args.store_in_db {
        meta::store_tables(&mut db, &args.metadata_table_name, &mut tables).await?;
    }

    if !args.skip_ts_generation {
        ts::write_tables(&args.output, &mut tables, args.store_in_db)?;
    }

    println!("\nAll operations done ✅");

    Ok(())
}
