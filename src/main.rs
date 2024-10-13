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
use std::iter;

use clap::Parser;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use surrealdb::sql::statements::{DefineFieldStatement, DefineTableStatement};
use surrealdb::sql::{self, Kind};
use surrealdb::sql::{statements::DefineStatement, Query, Statement};
use surrealdb::{engine::any::Any, opt::auth::Root, Surreal};

use surrealdb::syn::parser::Parser as SurrealParser;

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

    let mut db = surrealdb::engine::any::connect(&args.connection_url).await?;
    db.signin(Root {
        username: &args.username,
        password: &args.password,
    })
    .await?;
    db.use_ns(&args.namespace).use_db(&args.database).await?;

    let tables = get_tables_for_db(&mut db).await?;
    let table_metas: Vec<_> = tables
        .into_iter()
        .map(|table| TableMeta {
            name: table.table.name.to_string(),
            fields: get_field_metas(&table.fields, "".to_string()),
            comment: table.table.comment.map(|c| c.to_string()),
        })
        .collect();

    if !args.skip_ts_generation {
        ts::write_tables(&args.output, &table_metas, args.store_in_db)?;
    }

    if args.store_in_db {
        meta::store_tables(&mut db, &args.metadata_table_name, table_metas).await?;
    }

    println!("\nAll operations done ✅");

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

        println!("Processing table: {}", table.name);

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
    let info = info.expect(&format!("Failed to get information of table {table}."));

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableMeta {
    name: String,
    fields: Vec<FieldMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FieldMeta {
    name: String,
    r#type: FieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "name")]
enum FieldType {
    Any,
    Null,
    Boolean,
    String,
    Number,
    Date,
    Option {
        inner: Box<FieldType>,
    },
    Record {
        table: String,
    },
    Array {
        item: Box<FieldType>,
    },
    Object {
        fields: Option<Vec<FieldMeta>>,
    },
    Union {
        variants: Vec<FieldType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        kind: Option<EnumUnionKind>,
    },
    Literal(Literal),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
enum EnumUnionKind {
    String,
    Number,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
enum Literal {
    String { value: String },
    Number { value: f64 },
    Array { items: Vec<FieldType> },
}

impl Into<FieldType> for Literal {
    fn into(self) -> FieldType {
        FieldType::Literal(self)
    }
}

fn get_field_metas(fields: &Vec<DefineFieldStatement>, prefix: String) -> Vec<FieldMeta> {
    let mut field_metas = vec![];

    let mut fields = fields.into_iter();
    while let Some(DefineFieldStatement {
        name,
        kind,
        comment,
        ..
    }) = &fields.next()
    {
        let path = name.to_string();
        let name = path[prefix.len()..].to_string();

        field_metas.push(FieldMeta {
            name,
            r#type: get_field_type(path, kind.clone(), &mut fields),
            comment: comment.clone().map(|c| c.to_string()),
        });
    }

    field_metas
}

fn get_field_type<'a>(
    path: String,
    kind: Option<Kind>,
    fields: &mut (impl Iterator<Item = &'a DefineFieldStatement> + std::clone::Clone),
) -> FieldType {
    match kind {
        None => FieldType::Any,
        Some(kind) => match kind {
            Kind::Any => FieldType::Any,
            Kind::Null => FieldType::Null,
            Kind::Bool => FieldType::Boolean,
            Kind::Decimal | Kind::Float | Kind::Int | Kind::Number => FieldType::Number,
            Kind::String | Kind::Uuid | Kind::Duration => FieldType::String,
            Kind::Datetime => FieldType::Date,
            Kind::Option(kind) => {
                let inner = get_field_type(path, Some(*kind), fields);
                FieldType::Option { inner: inner.into() }
            },
            Kind::Object => {
                let prefix = format!("{path}.");

                let subfields: Vec<_> = fields
                    .take_while_ref(|f| f.name.to_string().starts_with(&prefix))
                    .cloned()
                    .collect();

                if subfields.len() == 0 {
                    FieldType::Object{ fields: None }
                } else {
                    let subfields = get_field_metas(&subfields, prefix);
                    FieldType::Object{ fields: Some(subfields) }
                }
            }
            Kind::Record(vec) => {
                let record_interface = vec[0].to_string();
                FieldType::Record{ table: record_interface }
            }
            Kind::Either(kinds) => {
                let variants: Vec<_> = kinds
                    .into_iter()
                    .map(|kind| get_field_type(path.clone(), Some(kind), fields))
                    .collect();

                FieldType::Union{ variants: variants.clone(), kind: get_union_enum_kind(variants) }
            }
            Kind::Set(inner, _) | Kind::Array(inner, _) => {
                let item = match fields.next() {
                    Some(item_definition) => {
                        get_field_type(
                            item_definition.name.to_string(),
                            item_definition.kind.clone(),
                            fields,
                        )
                    },
                    None => get_field_type(path, Some(*inner), fields)
                };

                FieldType::Array{ item: item.into() }
            }
            Kind::Literal(literal) => match literal {
                sql::Literal::String(value) => Literal::String{value: value[..].to_string()}.into(),
                sql::Literal::Number(number) => Literal::Number{value: number.as_float()}.into(),
                sql::Literal::Array(kinds) => {
                    let items: Vec<_> = kinds
                        .into_iter()
                        .map(|kind| get_field_type(path.clone(), Some(kind), fields))
                        .collect();

                    Literal::Array{ items }.into()
                },
                sql::Literal::Object(map) => {
                    let fields = map.into_iter().map(|(name, kind)| {
                        let field_type = get_field_type(format!("{path}.{name}"), Some(kind), &mut iter::empty());
                        FieldMeta {
                            name,
                            r#type: field_type,
                            comment: None,
                        }
                    })
                    .collect();

                    FieldType::Object{ fields: Some(fields) }
                },
                _ => unimplemented!(
                    "The type of field '{path}' is not yet supported. Please open an issue on github."
                ),
            },
            _ => unimplemented!(
                "The type of field '{path}' is not yet supported. Please open an issue on github."
            ),
        },
    }
}

fn get_union_enum_kind(variants: Vec<FieldType>) -> Option<EnumUnionKind> {
    if variants.is_empty() {
        return None;
    }

    if variants
        .iter()
        .all(|v| matches!(v, FieldType::Literal(Literal::String { .. })))
    {
        return Some(EnumUnionKind::String);
    }

    if variants
        .iter()
        .all(|v| matches!(v, FieldType::Literal(Literal::Number { .. })))
    {
        return Some(EnumUnionKind::Number);
    }

    None
}
