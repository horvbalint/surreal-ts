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
use std::fs::File;
use std::io::BufReader;
use std::iter;

use clap::{CommandFactory, Parser};
use config::Config;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use surrealdb::sql::statements::DefineFieldStatement;
use surrealdb::sql::{self, Kind};
use surrealdb::sql::{statements::DefineStatement, Query, Statement};
use surrealdb::{engine::any::Any, opt::auth::Root, Surreal};

use outputs::{db, ts::TSGenerator};
use surrealdb::syn::parser::Parser as SurrealParser;

mod config;
mod outputs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = Config::parse();
    if let Some(path) = &config.config_file_path {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        config = serde_json::from_reader(reader)?
    };

    let (Some(namespace), Some(database)) = (&config.namespace, &config.database) else {
        println!("No 'namespace' or 'database' provided in the config, see the help output for correct usage:\n");
        Config::command().print_help().ok();
        return Ok(());
    };

    let mut db = surrealdb::engine::any::connect(&config.address).await?;
    db.signin(Root {
        username: &config.username,
        password: &config.password,
    })
    .await?;
    db.use_ns(namespace).use_db(database).await?;

    let table_metas = get_tables_metas_for_db(&mut db).await?;

    if !config.skip_ts_generation {
        TSGenerator::new(&config).write_tables(&table_metas)?;
    }

    if config.store_meta_in_db {
        db::store_tables_in_db(&mut db, table_metas, &config).await?;
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

async fn get_tables_metas_for_db(db: &mut Surreal<Any>) -> anyhow::Result<TableMetas> {
    let mut tables = BTreeMap::new();

    let info: Option<DatabaseInfo> = db.query("INFO FOR DB").await?.take(0)?;
    let info = info.expect("Failed to get information of the database.");

    let every_table = info.tables.into_values().collect::<Vec<_>>().join(";\n");
    let result = parse_sql(&every_table);

    for stmt in result {
        let Statement::Define(DefineStatement::Table(table)) = stmt else {
            panic!("Database table list contained define statement for not table.")
        };

        println!("Processing table: {}", table.name);

        let fields = get_field_metas_for_table(db, &table.name).await?;
        let table_meta = TableMeta {
            fields: get_field_metas(&fields, "".to_string()),
            comment: table.comment.map(|c| c.to_string()),
        };

        tables.insert(table.name.to_string(), table_meta);
    }

    Ok(tables)
}

async fn get_field_metas_for_table(
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

type TableMetas = BTreeMap<String, TableMeta>;
type FieldMetas = BTreeMap<String, FieldMeta>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableMeta {
    fields: FieldMetas,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FieldMeta {
    r#type: FieldType,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    has_default: bool,
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
    Option { inner: Box<FieldType> },
    Record { table: String },
    Array { item: Box<FieldType> },
    Object { fields: Option<FieldMetas> },
    Union(Union),
    Literal(Literal),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
enum Union {
    Normal { variants: Vec<FieldType> },
    Enum(Enum),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "enum")]
enum Enum {
    String { variants: Vec<String> },
    Number { variants: Vec<f64> },
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

fn get_field_metas(fields: &Vec<DefineFieldStatement>, prefix: String) -> FieldMetas {
    let mut field_metas = BTreeMap::new();

    let mut fields = fields.into_iter();
    while let Some(field) = &fields.next() {
        let path = field.name.to_string();
        let name = path[prefix.len()..].to_string();

        let field_meta = FieldMeta {
            r#type: get_field_type(path, field.kind.clone(), &mut fields),
            has_default: field.default.is_some(),
            comment: field.comment.clone().map(|c| c.to_string()),
        };

        field_metas.insert(name, field_meta);
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

                FieldType::Union(get_union_variant(variants))
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
                        let field_meta = FieldMeta {
                            r#type: field_type,
                            has_default: false,
                            comment: None,
                        };

                        (name, field_meta)
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

fn get_union_variant(variants: Vec<FieldType>) -> Union {
    let strings: Vec<_> = variants
        .iter()
        .filter_map(|v| match v {
            FieldType::Literal(Literal::String { value }) => Some(value.clone()),
            _ => None,
        })
        .collect();

    if strings.len() == variants.len() {
        return Union::Enum(Enum::String { variants: strings });
    }

    let numbers: Vec<_> = variants
        .iter()
        .filter_map(|v| match v {
            FieldType::Literal(Literal::Number { value }) => Some(value.clone()),
            _ => None,
        })
        .collect();

    if numbers.len() == variants.len() {
        return Union::Enum(Enum::Number { variants: numbers });
    }

    Union::Normal { variants }
}
