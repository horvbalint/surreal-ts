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

use core::panic;
use std::collections::BTreeMap;

use clap::Parser;
use serde::Deserialize;
use surrealdb::sql::statements::{DefineFieldStatement, DefineTableStatement};
use surrealdb::sql::{statements::DefineStatement, Query, Statement};
use surrealdb::{engine::any::Any, opt::auth::Root, Surreal};

use surrealdb::syn::parser::Parser as SurrealParser;

// mod meta;
// mod parser;
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

    let tables = get_tables_for_db(&mut db).await?;
    //     .await?
    //     .into_iter()
    //     .filter(|(name, _)| name != &args.metadata_table_name)
    //     .collect();

    // println!();

    // if args.store_in_db {
    //     meta::store_tables(&mut db, &args.metadata_table_name, &mut tables).await?;
    // }

    if !args.skip_ts_generation {
        ts::write_tables(&args.output, &tables, args.store_in_db)?;
    }

    // println!("\nAll operations done ✅");

    Ok(())
}

#[derive(Deserialize, Debug)]
struct DatabaseInfo {
    tables: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct TableInfo {
    fields: BTreeMap<String, String>,
}

#[derive(Debug)]
struct Table {
    table: DefineTableStatement,
    fields: Vec<DefineFieldStatement>,
}

async fn get_tables_for_db(db: &mut Surreal<Any>) -> anyhow::Result<Vec<Table>> {
    let mut tables = vec![];

    let info: Option<DatabaseInfo> = db.query("INFO FOR DB").await?.take(0)?;
    let info = info.expect("Failed to get information of the database.");

    let every_table = info.tables.into_values().collect::<Vec<_>>().join(";\n");
    let result = parse_sql(&every_table);

    for stmt in result {
        let Statement::Define(DefineStatement::Table(table)) = stmt else {
            panic!("Database table list contained define statement for not table.")
        };

        let fields = get_fields_for_table(db, &table.name).await?;

        tables.push(Table { table, fields });
    }

    Ok(tables)
}

async fn get_fields_for_table(
    db: &mut Surreal<Any>,
    table: &str,
) -> anyhow::Result<Vec<DefineFieldStatement>> {
    let mut fields = vec![];

    let info: Option<TableInfo> = db.query(format!("INFO FOR TABLE {table}")).await?.take(0)?;
    let info = info.expect("Failed to get information of the database.");

    let every_field = info.fields.into_values().collect::<Vec<_>>().join(";\n");
    let result = parse_sql(&every_field);

    for stmt in result {
        let Statement::Define(DefineStatement::Field(field)) = stmt else {
            panic!("The field list of table '{table}' contained define statement for not field.")
        };

        fields.push(field);
    }

    Ok(fields)
}

fn parse_sql(sql: &str) -> Query {
    let mut parser = SurrealParser::new(sql.as_bytes());

    let mut stack = reblessive::Stack::new();
    let result = stack.enter(|ctx| parser.parse_query(ctx)).finish().unwrap();

    result
}
