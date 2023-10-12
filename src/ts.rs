use std::fs::File;
use std::io::Write;

use crate::Tables;
use crate::{Field, utils::create_interface_name, Fields, FieldPayload};

pub fn write_tables(output_path: &str, tables: &mut Tables, add_table_meta_types: bool) -> anyhow::Result<()> {
    println!("Writing type declaration file...");
    let mut file = File::create(output_path)?;

    if add_table_meta_types {
        write!(&mut file,
"export type TableMeta = {{
  name: string
  fields: FieldMeta[]
  comment?: string
}}

export type FieldMeta = {{
  name: string
  isOptional: boolean
  isArray: boolean
  type: string
  comment?: string
  isRecord?: true
  fields?: FieldMeta[]
}}\n\n"
        )?;
    }

    for (name, table) in tables.iter_mut() {
        write_table(&mut file, name, &mut table.fields, false)?;
        write_table(&mut file, name, &mut table.fields, true)?;
    }

    Ok(())
}

fn write_table(file: &mut File, name: &str, fields: &mut Fields, from_db: bool) -> anyhow::Result<()> {
    let interface_name = create_interface_name(name, from_db);
    write!(file, "export type {interface_name} = ")?;

    fields.insert("id".to_string(), Field {
        is_optional: !from_db,
        is_array: false,
        comment: None,
        payload: FieldPayload::Type {
            name: "string".to_string(),
            is_record: false
        }
    });

    write_object(file, fields, from_db, 0)?;

    write!(file, "\n\n")?;
    Ok(())
}

fn write_object(file: &mut File, fields: &Fields, from_db: bool, depth: usize) -> anyhow::Result<()> {
    if fields.len() == 0 {
        write!(file, "object")?;
    } else {
        write!(file, "{{\n")?;

        let indentation = "\t".repeat(depth);
        for key in fields.keys() {
            write!(file, "{indentation}\t{key}")?;
            write_field(file, &fields[key], from_db, depth)?;
        }

        write!(file, "{indentation}}}")?;
    }
    Ok(())
}

fn write_field(file: &mut File, field: &Field, from_db: bool, depth: usize) -> anyhow::Result<()> {
    if field.is_optional {
        write!(file, "?")?;
    }

    write!(file, ": ")?;

    if field.is_array {
        write!(file, "Array<")?;
    }

    match &field.payload {
        FieldPayload::Type {name, is_record} => {
            write_type(file, name, *is_record, from_db)?;
        },
        FieldPayload::SubFields(fields) => {
            write_object(file, fields, from_db, depth+1)?;
        }
    }

    if field.is_array {
        write!(file, ">")?;
    }

    write!(file, "\n")?;
    Ok(())
}

fn write_type(file: &mut File, name: &String, is_record: bool, from_db: bool) -> anyhow::Result<()> {
    let name = if is_record {
        let ref_name = create_interface_name(name, from_db);

        if from_db {
            format!("{ref_name}['id'] | {ref_name}")
        } else {
            format!("Required<{ref_name}>['id']")
        }
    } else if name == "datetime" {
        if from_db {
            "string".to_string()
        } else {
            "Date | string".to_string()
        }
    } else if name == "bool" {
        "boolean".to_string()
    } else if name == "decimal" || name == "float" || name == "int" {
        "number".to_string()
    } else if name == "duration" || name == "geometry" {
        "string".to_string()
    } else if name == "array" || name == "set" { // we get here when array or set is used without generic type parameter
        "[]".to_string()
    } else {
  name.to_string()
    };

    write!(file, "{name}")?;
    Ok(())
}