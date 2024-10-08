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

use anyhow::Error;
use clap::Parser;
use serde::Deserialize;
use surrealdb::{engine::any::Any, opt::auth::Root, Surreal};

use surrealdb::syn::parser::Parser as SurrealParser;

mod meta;
mod parser;
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

#[derive(Deserialize, Debug)]
struct DatabaseInfo {
    tables: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct TableInfo {
    fields: BTreeMap<String, String>,
}

struct Generator {
    tables: Tables,
}

impl Generator {
    pub async fn process(db: &mut Surreal<Any>) -> anyhow::Result<Tables> {
        let mut generator = Self {
            tables: BTreeMap::new(),
        };

        let info: Option<DatabaseInfo> = db.query("INFO FOR DB").await?.take(0)?;
        let info = info.expect("Failed to get information of the database.");

        for (name, definition) in info.tables {
            generator.process_table(db, &name, &definition).await?
        }

        Ok(generator.tables)
    }

    async fn process_table(
        &mut self,
        db: &mut Surreal<Any>,
        name: &str,
        definition: &str,
    ) -> anyhow::Result<()> {
        println!("Processing table: {name}");

        let info: Option<TableInfo> = db.query(format!("INFO FOR TABLE {name}")).await?.take(0)?;
        let info = info.expect("Failed to get information of the table.");

        let every_field = info.fields.into_values().collect::<Vec<_>>().join(";\n");
        println!("{every_field:#?}");

        let mut parser = SurrealParser::new(every_field.as_bytes());

        let mut stack = reblessive::Stack::new();
        let result = stack.enter(|ctx| parser.parse_query(ctx)).finish().unwrap();

        dbg!(result);

        // let (_, comment) = parser::parse_comment(definition).map_err(|err| err.to_owned())?;

        // let table = self.tables.entry(name.to_string()).or_insert(FieldTree {
        //     is_optional: false,
        //     is_array: false,
        //     comment,
        //     r#type: FieldType::Node(Node {
        //         fields: BTreeMap::new(),
        //     }),
        // });

        // for path in info.fields.keys() {
        //     Self::process_field(table, path, &info.fields[path])?;
        // }

        Ok(())
    }

    fn process_field(tree: &mut FieldTree, path: &str, definition: &str) -> anyhow::Result<()> {
        let field = FieldTree::from(definition)?;
        let normalized_path = path.replace("[*]", ""); // removing array item decorators, since they are not separate fields

        if path.ends_with("[*]") {
            Self::handle_array_item(tree, &normalized_path, field)?
        } else {
            tree.insert(&normalized_path, field)?;
        }

        Ok(())
    }

    fn handle_array_item(
        tree: &mut FieldTree,
        parent_path: &str,
        field: FieldTree,
    ) -> anyhow::Result<()> {
        let parent = tree
            .get_mut(parent_path)
            .ok_or(Error::msg("Array item reached before the array"))?;

        match &mut parent.r#type {
            FieldType::Leaf(parent_leaf) => match field.r#type {
                FieldType::Leaf(item_leaf) => {
                    parent.is_array = true;
                    parent_leaf.name = item_leaf.name;
                    parent_leaf.is_record = item_leaf.is_record;
                }
                FieldType::Node(_) => {
                    *parent = FieldTree {
                        is_array: true,
                        ..field
                    }
                }
            },
            FieldType::Node(_) => {
                dbg!(field);
                dbg!(parent);
                // Using the [*] operator on objects does not seem valid
                unimplemented!("If you encounter this message, please open an issue at: https://github.com/horvbalint/surreal-ts/issues");
            }
        }

        Ok(())
    }
}

type Tables = BTreeMap<String, FieldTree>;
type Fields = BTreeMap<String, FieldTree>;

#[derive(Debug)]
pub struct FieldTree {
    is_optional: bool,
    is_array: bool,
    comment: Option<String>,
    r#type: FieldType,
}

#[allow(dead_code)]
impl FieldTree {
    pub fn from(definition: &str) -> anyhow::Result<Self> {
        let (remaining, raw_type) =
            parser::parse_type_from_definition(definition).map_err(|err| err.to_owned())?;

        let (_, props) = parser::parse_type(raw_type).map_err(|err| err.to_owned())?;
        let (_, comment) = parser::parse_comment(remaining).map_err(|err| err.to_owned())?;

        let field = Self {
            is_array: props.is_array,
            is_optional: props.is_optional,
            comment,
            r#type: if props.name == "object" {
                FieldType::Node(Node {
                    fields: BTreeMap::new(),
                })
            } else {
                FieldType::Leaf(Leaf {
                    name: props.name.to_string(),
                    is_record: props.is_record,
                })
            },
        };

        Ok(field)
    }

    fn insert(&mut self, path: &str, field: FieldTree) -> anyhow::Result<()> {
        let (parent, key) = match path.rsplit_once('.') {
            Some((parent_path, last_step)) => {
                let parent = self.get_mut(parent_path).ok_or(anyhow::Error::msg(
                    "One of the parents is missing from the tree",
                ))?;

                (parent, last_step)
            }
            None => (self, path),
        };

        parent
            .r#type
            .as_node_mut()
            .fields
            .insert(key.to_string(), field);

        Ok(())
    }

    fn get_mut(&mut self, path: &str) -> Option<&mut FieldTree> {
        let mut cursor = self;

        for step in path.split('.') {
            let FieldType::Node(node) = &mut cursor.r#type else {
                return None;
            };

            cursor = node.fields.get_mut(step)?
        }

        Some(cursor)
    }
}

#[derive(Debug)]
pub enum FieldType {
    Node(Node),
    Leaf(Leaf),
}

impl FieldType {
    fn as_node(&self) -> &Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldType::Leaf as FieldType::Node"),
        }
    }

    fn as_node_mut(&mut self) -> &mut Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldType::Leaf as FieldType::Node"),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    fields: Fields,
}

#[derive(Debug)]
pub struct Leaf {
    name: String,
    is_record: bool,
}
