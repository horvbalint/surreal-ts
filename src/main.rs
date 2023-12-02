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
use regex::Regex;
use serde::Deserialize;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
    opt::auth::Root,
    Surreal,
};

mod meta;
mod parser;
mod ts;
mod utils;
/// A simple typescript definition generator for SurrealDB
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// The connection url to the SurrealDB instance
    #[arg(short, long, default_value = "localhost:8000")]
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

    let mut db = connect_to_db(&args.connection_url).await?;

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

    print!("\n");

    if args.store_in_db {
        meta::store_tables(&mut db, &args.metadata_table_name, &mut tables).await?;
    }

    if !args.skip_ts_generation {
        ts::write_tables(&args.output, &mut tables, args.store_in_db)?;
    }

    println!("\nAll operations done ✅");

    Ok(())
}

async fn connect_to_db(connection_url: &str) -> anyhow::Result<Surreal<Client>> {
    let re = Regex::new("^.+://")?;
    let url = re.replace(connection_url, "").to_string();

    let db = if connection_url.starts_with("https://") || connection_url.starts_with("wss://") {
        Surreal::new::<Wss>(url).await?
    } else {
        Surreal::new::<Ws>(url).await?
    };

    Ok(db)
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
    pub async fn process(db: &mut Surreal<Client>) -> anyhow::Result<Tables> {
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
        db: &mut Surreal<Client>,
        name: &str,
        definition: &str,
    ) -> anyhow::Result<()> {
        println!("Processing table: {name}");

        let info: Option<TableInfo> = db
            .query(format!("INFO FOR TABLE {name}"))
            .bind(("table", name))
            .await?
            .take(0)?;
        let info = info.expect("Failed to get information of the table.");

        let (_, comment) = parser::parse_comment(definition).map_err(|err| err.to_owned())?;

        let mut table = self
            .tables
            .entry(name.to_string())
            .or_insert(FieldTree::Node(Node {
                is_optional: false,
                is_array: false,
                comment,
                fields: BTreeMap::new(),
            }));

        for path in info.fields.keys() {
            Self::process_field(&mut table, path, &info.fields[path])?;
        }

        Ok(())
    }

    fn process_field(tree: &mut FieldTree, path: &str, definition: &str) -> anyhow::Result<()> {
        let field = FieldTree::from(&definition)?;

        if path.ends_with("[*]") {
            let parent_path = &path[..path.len() - 3];
            let parent = tree.get_mut(parent_path).ok_or(Error::msg(
                "Array item descriptor was reached before the array descriptor",
            ))?;

            match parent {
                FieldTree::Leaf(parent_leaf) => match field {
                    FieldTree::Leaf(item_leaf) => {
                        parent_leaf.is_array = true;
                        parent_leaf.name = item_leaf.name;
                        parent_leaf.is_record = item_leaf.is_record;
                    }
                    FieldTree::Node(_) => {
                        *parent = FieldTree::Node(Node {
                            is_array: true,
                            ..field.to_node()
                        })
                    }
                },
                FieldTree::Node(_) => {
                    // Using the [*] opencounter this message, please open an issue at: https://github.com/horvbalint/surreal-ts/issues");
                }
            }
        } else {
            let path = path.replace("[*]", "");
            tree.insert(&path, field)?;
        }

        Ok(())
    }
}

type Tables = BTreeMap<String, FieldTree>;
type Fields = BTreeMap<String, FieldTree>;

#[derive(Debug)]
pub struct Node {
    is_optional: bool,
    is_array: bool,
    comment: Option<String>,
    fields: Fields,
}

#[derive(Debug)]
pub struct Leaf {
    is_optional: bool,
    is_array: bool,
    comment: Option<String>,
    name: String,
    is_record: bool,
}

#[derive(Debug)]
pub enum FieldTree {
    Node(Node),
    Leaf(Leaf),
}

#[allow(dead_code)]
pub struct FieldCommon<'a> {
    is_optional: bool,
    is_array: bool,
    comment: &'a Option<String>,
}

#[allow(dead_code)]
impl FieldTree {
    pub fn from(definition: &str) -> anyhow::Result<Self> {
        let (remaining, raw_type) =
            parser::parse_type_from_definition(definition).map_err(|err| err.to_owned())?;

        let (_, props) = parser::parse_type(raw_type).map_err(|err| err.to_owned())?;
        let (_, comment) = parser::parse_comment(remaining).map_err(|err| err.to_owned())?;

        let field = if props.name == "object" {
            Self::Node(Node {
                is_array: props.is_array,
                is_optional: props.is_optional,
                comment,
                fields: BTreeMap::new(),
            })
        } else {
            Self::Leaf(Leaf {
                is_array: props.is_array,
                is_optional: props.is_optional,
                comment,
                name: props.name.to_string(),
                is_record: props.is_record,
            })
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

        parent.as_node_mut().fields.insert(key.to_string(), field);

        Ok(())
    }

    fn get_mut(&mut self, path: &str) -> Option<&mut FieldTree> {
        let mut cursor = self;

        for step in path.split('.') {
            let Self::Node(Node { fields, .. }) = cursor else {
                return None;
            };

            let Some(field) = fields.get_mut(step) else {
                return None;
            };

            cursor = field
        }

        Some(cursor)
    }

    fn to_leaf(self) -> Leaf {
        match self {
            Self::Leaf(leaf) => leaf,
            Self::Node(_) => panic!("Tried to use FieldTree::Node as FieldTree::Leaf"),
        }
    }

    fn as_leaf(&self) -> &Leaf {
        match self {
            Self::Leaf(leaf) => leaf,
            Self::Node(_) => panic!("Tried to use FieldTree::Node as FieldTree::Leaf"),
        }
    }

    fn as_leaf_mut(&mut self) -> &mut Leaf {
        match self {
            Self::Leaf(leaf) => leaf,
            Self::Node(_) => panic!("Tried to use FieldTree::Node as FieldTree::Leaf"),
        }
    }

    fn to_node(self) -> Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldTree::Leaf as FieldTree::Node"),
        }
    }

    fn as_node(&self) -> &Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldTree::Leaf as FieldTree::Node"),
        }
    }

    fn as_node_mut(&mut self) -> &mut Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldTree::Leaf as FieldTree::Node"),
        }
    }

    fn get_common(&self) -> FieldCommon {
        match self {
            FieldTree::Node(node) => FieldCommon {
                is_optional: node.is_optional,
                is_array: node.is_array,
                comment: &node.comment,
            },
            FieldTree::Leaf(leaf) => FieldCommon {
                is_optional: leaf.is_optional,
                is_array: leaf.is_array,
                comment: &leaf.comment,
            },
        }
    }
}
