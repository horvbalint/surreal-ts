use core::panic;
use std::collections::BTreeMap;

use serde::Deserialize;
use surrealdb::{engine::any::Any, Surreal};
use thiserror::Error;

use crate::parser;

#[derive(Deserialize, Debug)]
pub struct DatabaseInfo {
    tables: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct TableInfo {
    fields: BTreeMap<String, String>,
}

pub struct Generator {
    tables: Tables,
}

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Querying the database was unsuccessful")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("Failed to parse a Surql statement")]
    ParsingError(#[from] nom::Err<nom::error::Error<String>>),
    #[error("Failed to process one of the tables field")]
    FieldProcessError(#[from] FieldTreeError),
    #[error("Array item reached before the array")]
    ArrayProcessError,
}

impl Generator {
    pub async fn process(db: &mut Surreal<Any>) -> Result<Tables, GeneratorError> {
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
    ) -> Result<(), GeneratorError> {
        println!("Processing table: {name}");

        let info: Option<TableInfo> = db
            .query(format!("INFO FOR TABLE {name}"))
            .bind(("table", name))
            .await?
            .take(0)?;
        let info = info.expect("Failed to get information of the table.");

        let (_, comment) = parser::parse_comment(definition).map_err(|err| err.to_owned())?;

        let table = self.tables.entry(name.to_string()).or_insert(FieldTree {
            is_optional: false,
            is_array: false,
            comment,
            r#type: FieldType::Node(Node {
                fields: BTreeMap::new(),
            }),
        });

        for path in info.fields.keys() {
            Self::process_field(table, path, &info.fields[path])?;
        }

        Ok(())
    }

    fn process_field(
        tree: &mut FieldTree,
        path: &str,
        definition: &str,
    ) -> Result<(), GeneratorError> {
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
    ) -> Result<(), GeneratorError> {
        let parent = tree
            .get_mut(parent_path)
            .ok_or(GeneratorError::ArrayProcessError)?;

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
                // Using the [*] operator on objects does not seem valid
                unimplemented!("If you encounter this message, please open an issue at: https://github.com/horvbalint/surreal-ts/issues");
            }
        }

        Ok(())
    }
}

pub type Tables = BTreeMap<String, FieldTree>;
pub type Fields = BTreeMap<String, FieldTree>;

#[derive(Debug, Error)]
pub enum FieldTreeError {
    #[error("Failed to parse a field definition surql statement")]
    ParsingError(#[from] nom::Err<nom::error::Error<String>>),
    #[error("One of the parents is missing from the tree")]
    InsertError,
}

#[derive(Debug)]
pub struct FieldTree {
    pub is_optional: bool,
    pub is_array: bool,
    pub comment: Option<String>,
    pub r#type: FieldType,
}

#[allow(dead_code)]
impl FieldTree {
    pub fn from(definition: &str) -> Result<Self, FieldTreeError> {
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

    fn insert(&mut self, path: &str, field: FieldTree) -> Result<(), FieldTreeError> {
        let (parent, key) = match path.rsplit_once('.') {
            Some((parent_path, last_step)) => {
                let parent = self
                    .get_mut(parent_path)
                    .ok_or(FieldTreeError::InsertError)?;

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
    pub fn as_node(&self) -> &Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldType::Leaf as FieldType::Node"),
        }
    }

    pub fn as_node_mut(&mut self) -> &mut Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldType::Leaf as FieldType::Node"),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub fields: Fields,
}

#[derive(Debug)]
pub struct Leaf {
    pub name: String,
    pub is_record: bool,
}
