mod tests;

use std::collections::BTreeMap;
use thiserror::Error;

use crate::parser;

pub type Fields<'a> = BTreeMap<&'a str, FieldTree<'a>>;

#[derive(Debug, Error, PartialEq)]
pub enum FieldTreeError {
    #[error("Failed to parse a field definition surql statement")]
    ParsingError(#[from] nom::Err<nom::error::Error<String>>),
    #[error("One of the parents is missing from the tree")]
    InsertError,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct FieldTree<'a> {
    pub is_optional: bool,
    pub is_array: bool,
    pub comment: Option<&'a str>,
    pub r#type: FieldType<'a>,
}

#[allow(dead_code)]
impl<'a> FieldTree<'a> {
    pub fn from(definition: &'a str) -> Result<Self, FieldTreeError> {
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
                    name: props.name,
                    is_record: props.is_record,
                })
            },
        };

        Ok(field)
    }

    pub fn insert(&mut self, path: &'a str, field: FieldTree<'a>) -> Result<(), FieldTreeError> {
        let (parent, key) = match path.rsplit_once('.') {
            Some((parent_path, last_step)) => {
                let parent = self
                    .get_mut(parent_path)
                    .ok_or(FieldTreeError::InsertError)?;

                (parent, last_step)
            }
            None => (self, path),
        };

        parent.r#type.as_node_mut().fields.insert(key, field);

        Ok(())
    }

    pub fn get_mut<'b>(&'b mut self, path: &str) -> Option<&'b mut FieldTree<'a>> {
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum FieldType<'a> {
    Node(Node<'a>),
    Leaf(Leaf<'a>),
}

impl<'a> FieldType<'a> {
    pub fn as_node(&self) -> &Node {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldType::Leaf as FieldType::Node"),
        }
    }

    pub fn as_node_mut(&mut self) -> &mut Node<'a> {
        match self {
            Self::Node(node) => node,
            Self::Leaf(_) => panic!("Tried to use FieldType::Leaf as FieldType::Node"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone)]
pub struct Node<'a> {
    pub fields: Fields<'a>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Leaf<'a> {
    pub name: &'a str,
    pub is_record: bool,
}
