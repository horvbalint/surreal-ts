use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Connection, Surreal};

use surreal_ts_core::{FieldTree, FieldType, Fields, Tables};

pub async fn store_tables<C: Connection>(
    db: &mut Surreal<C>,
    metadata_table_name: &str,
    tables: &mut Tables,
) -> anyhow::Result<()> {
    println!("Writing table metadata into database...");

    db.query(format!(
        "
        REMOVE TABLE {metadata_table_name};
        DEFINE TABLE {metadata_table_name} SCHEMALESS
            PERMISSIONS
                FOR select FULL;
    "
    ))
    .await?;

    for (name, table) in tables {
        store_table(db, metadata_table_name, table, name).await?;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TableMeta<'a> {
    name: &'a str,
    fields: Vec<FieldMeta<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FieldMeta<'a> {
    name: &'a str,
    is_optional: bool,
    is_array: bool,
    r#type: &'a str,
    #[serde(flatten)]
    discriminating: DiscriminatingFieldParts<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum DiscriminatingFieldParts<'a> {
    #[serde(rename_all = "camelCase")]
    SubFields {
        fields: Vec<FieldMeta<'a>>,
    },
    #[serde(rename_all = "camelCase")]
    Record {
        is_record: bool,
    },
    None {},
}

async fn store_table<'a, C: Connection>(
    db: &mut Surreal<C>,
    metadata_table_name: &str,
    table: &FieldTree<'a>,
    name: &str,
) -> anyhow::Result<()> {
    let table_meta = TableMeta {
        name,
        comment: table.comment.clone(),
        fields: get_fields(&table.r#type.as_node().fields),
    };

    db.create::<Option<Record>>((metadata_table_name, name))
        .content(table_meta)
        .await?;

    Ok(())
}

fn get_fields<'a>(fields: &'a Fields) -> Vec<FieldMeta<'a>> {
    fields
        .iter()
        .map(|(name, field)| FieldMeta {
            name,
            is_optional: field.is_optional,
            is_array: field.is_array,
            comment: field.comment.clone(),
            r#type: calc_field_type(field),
            discriminating: calc_discriminating_parts(field),
        })
        .collect()
}

fn calc_field_type<'a>(field: &'a FieldTree) -> &'a str {
    match &field.r#type {
        FieldType::Node(_) => "object",
        FieldType::Leaf(leaf) => leaf.name,
    }
}

fn calc_discriminating_parts<'a>(field: &'a FieldTree) -> DiscriminatingFieldParts<'a> {
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
