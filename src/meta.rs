use surrealdb::{engine::any::Any, Surreal};

use crate::{config::Config, TableMeta};

pub async fn store_tables(
    db: &mut Surreal<Any>,
    tables: Vec<TableMeta>,
    config: &Config,
) -> anyhow::Result<()> {
    println!("Writing table metadata into database...");
    let metadata_table_name = &config.metadata_table_name;

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
