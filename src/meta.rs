use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{FieldTree, Fields, Tables};

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
        comment: table.as_node().comment.clone(),
        fields: get_fields(&table.as_node().fields),
    };

    db.create::<Option<TableMeta>>((metadata_table_name, name))
        .content(table_meta)
        .await?;

    Ok(())
}

fn get_fields(fields: &Fields) -> Vec<FieldMeta> {
    let mut field_metas = vec![];

    for (name, field) in fields {
        let meta = match field {
            FieldTree::Node(node) => FieldMeta {
                name: name.to_string(),
                is_optional: node.is_optional,
                is_array: node.is_array,
                r#type: "object".to_string(),
                comment: node.comment.clone(),
                discriminating: DiscriminatingFieldParts::SubFields {
                    fields: get_fields(&node.fields),
                },
            },
            FieldTree::Leaf(leaf) => FieldMeta {
                name: name.to_string(),
                is_optional: leaf.is_optional,
                is_array: leaf.is_array,
                r#type: leaf.name.clone(),
                comment: leaf.comment.clone(),
                discriminating: if leaf.is_record {
                    DiscriminatingFieldParts::Record { is_record: true }
                } else {
                    DiscriminatingFieldParts::None {}
                },
            },
        };

        field_metas.push(meta)
    }

    field_metas
}
