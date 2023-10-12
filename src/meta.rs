use serde::{Serialize, Deserialize};
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::{Fields, FieldPayload, Tables, Table};

pub async fn store_tables(db: &mut Surreal<Client>, metadata_table_name: &str, tables: &mut Tables) -> anyhow::Result<()> {
    println!("Writing table metadata into database...");

    db.query(format!("
        REMOVE TABLE {metadata_table_name};
        DEFINE TABLE {metadata_table_name} SCHEMALESS
            PERMISSIONS
                FOR create, update, delete NONE;
    "))
    .await?;

    for (name, table) in tables {
        store_table(db, &metadata_table_name, &table, name).await?;
    }
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableMeta {
    name: String,
    fields: Vec<FieldMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>
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
    None {}
}

async fn store_table(db: &mut Surreal<Client>, metadata_table_name: &str, table: &Table, name: &str) -> anyhow::Result<()> {
    let table_meta = TableMeta {
        name: name.to_string(),
        comment: table.comment.clone(),
        fields: get_fields(&table.fields),
    };

    db.create::<Option<TableMeta>>((metadata_table_name, name)).content(table_meta).await?;

    Ok(())
}

fn get_fields(fields: &Fields) -> Vec<FieldMeta> {
    let mut field_metas = vec![];

    for (name, field) in fields {
        let meta = FieldMeta {
            name: name.to_string(),
            is_optional: field.is_optional,
            is_array: field.is_array,
            r#type: get_type(&field.payload),
            comment: field.comment.clone(),
            discriminating: get_discriminating_parts(&field.payload),
        };

        field_metas.push(meta)
    }

    field_metas
}

fn get_type(payload: &FieldPayload) -> String {
    match payload {
        FieldPayload::Type { name, .. } => name.to_string(),
        FieldPayload::SubFields(_) => "object".to_string(),
    }
}

fn get_discriminating_parts(payload: &FieldPayload) -> DiscriminatingFieldParts {
    match payload {
        FieldPayload::Type { is_record, .. } if *is_record == true => {
            DiscriminatingFieldParts::Record {
                is_record: true
            }
        },
        FieldPayload::SubFields(fields) => {
            DiscriminatingFieldParts::SubFields {
                fields: get_fields(fields)
            }
        },
        _ => DiscriminatingFieldParts::None {}
    }
}