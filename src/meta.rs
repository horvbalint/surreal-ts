use surrealdb::{engine::any::Any, Surreal};

use crate::TableMeta;

pub async fn store_tables(
    db: &mut Surreal<Any>,
    metadata_table_name: &str,
    tables: Vec<TableMeta>,
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

    for table_meta in tables {
        db.create::<Option<TableMeta>>((metadata_table_name, table_meta.name.clone()))
            .content(table_meta)
            .await?;
    }

    Ok(())
}
