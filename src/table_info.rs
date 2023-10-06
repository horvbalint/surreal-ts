use serde::{Serialize, Deserialize};
use surrealdb::{Surreal, engine::remote::ws::Client};

use crate::{Fields, FieldPayload, Tables, Table};

pub async fn store_tables(db: &mut Surreal<Client>, tables: &mut Tables) -> anyhow::Result<()> {
    println!("Writing table infos into database...");

    db.query("
        REMOVE TABLE table_info;
        DEFINE TABLE table_info SCHEMALESS
            PERMISSIONS
                FOR create, update, delete NONE;
    ").await?;

    for (name, table) in tables {
        store_table(db, &table, name).await?;
    }
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TableInfo {
    name: String,
    fields: Vec<FieldInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FieldInfo {
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
        fields: Vec<FieldInfo>,
    },
    #[serde(rename_all = "camelCase")]
    Record {
        is_record: bool,
    },
    None {}
}

async fn store_table(db: &mut Surreal<Client>, table: &Table, name: &str) -> anyhow::Result<()> {
    let table_info = TableInfo {
        name: name.to_string(),
        comment: table.comment.clone(),
        fields: get_fields(&table.fields),
    };

    db.create::<Option<TableInfo>>(("table_info", name)).content(table_info).await?;

    Ok(())
}

fn get_fields(fields: &Fields) -> Vec<FieldInfo> {
    let mut field_infos = vec![];

    for (name, field) in fields {
        let info = FieldInfo {
            name: name.to_string(),
            is_optional: field.is_optional,
            is_array: field.is_array,
            r#type: get_type(&field.payload),
            comment: field.comment.clone(),
            discriminating: get_discriminating_parts(&field.payload),
        };

        field_infos.push(info)
    }

    field_infos
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