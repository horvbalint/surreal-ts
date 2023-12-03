use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{FieldTree, FieldType, Fields, Tables};

pub async fn store_tables(
    db: &mut Surreal<Client>,
    metadata_table_name: &str,
    tables: &mut Tables,
) -> anyhow::Result<()> {
    println!("Writing table metadata into database...");

    db.query(format!(
        "
        REMOVE TABLE {metadata_table_name};
        DEFINE TABLE {metadata_table_name} SCHEMALESS
            PERMISSIONS
                FOR create, update, delete NONE;
    "
    ))
    .await?;

    for (name, table) in tables {
        store_table(db, metadata_table_name, table, name).await?;
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableMeta {
    name: String,
    fields: Vec<FieldMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FieldMeta {
    name: String,
    is_optional: bool,
    is_array: bool,
    r#type: String,
    #[serde(flatten)]
    discriminating: DiscriminatingFieldParts,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum DiscriminatingFieldParts {
    #[serde(rename_all = "camelCase")]
    SubFields {
        fields: Vec<FieldMeta>,
    },
    #[serde(rename_all = "camelCase")]
    Record {
        is_record: bool,
    },
    None {},
}

async fn store_table(
    db: &mut Surreal<Client>,
    metadata_table_name: &str,
    table: &FieldTree,
    name: &str,
) -> anyhow::Result<()> {
    let table_meta = TableMeta {
        name: name.to_string(),
        comment: table.comment.clone(),
        fields: get_fields(&table.r#type.as_node().fields),
    };

    db.create::<Option<TableMeta>>((metadata_table_name, name))
        .content(table_meta)
        .await?;

    Ok(())
}

fn get_fields(fields: &Fields) -> Vec<FieldMeta> {
    fields
        .iter()
        .map(|(name, field)| FieldMeta {
            name: name.to_string(),
            is_optional: field.is_optional,
            is_array: field.is_array,
            comment: field.comment.clone(),
            r#type: calc_field_type(field),
            discriminating: calc_discriminating_parts(field),
        })
        .collect()
}

fn calc_field_type(field: &FieldTree) -> String {
    match &field.r#type {
        FieldType::Node(_) => "object".to_string(),
        FieldType::Leaf(leaf) => leaf.name.to_string(),
    }
}

fn calc_discriminating_parts(field: &FieldTree) -> DiscriminatingFieldParts {
    match &field.r#type {
        FieldType::Node(node) => DiscriminatingFieldParts::SubFields {
            fields: get_fields(&node.fields),
        },
        FieldType::Leaf(leaf) => {
            if leaf.is_record {
                DiscriminatingFieldParts::Record { is_record: true }
            } else {
                DiscriminatingFieldParts::None {}
            }
        }
    }
}
