use std::fs::File;
use std::io::Write;

use convert_case::{Case, Casing};

use surreal_ts_core::{FieldTree, FieldType, Fields, Leaf, Tables};

pub fn write_tables(
    output_path: &str,
    tables: &mut Tables,
    add_table_meta_types: bool,
) -> anyhow::Result<()> {
    println!("Writing type declaration file...");
    let mut file = File::create(output_path)?;

    if add_table_meta_types {
        write!(
            &mut file,
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
        let node = table.r#type.as_node_mut();

        write_table(&mut file, name, &mut node.fields, false)?;
        write_table(&mut file, name, &mut node.fields, true)?;
    }

    Ok(())
}

fn write_table(
    file: &mut File,
    name: &str,
    fields: &mut Fields,
    from_db: bool,
) -> anyhow::Result<()> {
    let interface_name = create_interface_name(name, from_db);
    write!(file, "export type {interface_name} = ")?;

    let field = FieldTree {
        is_optional: !from_db,
        is_array: false,
        comment: None,
        r#type: FieldType::Leaf(Leaf {
            name: "string".to_string(),
            is_record: false,
        }),
    };
    fields.insert("id".to_string(), field);

    write_object(file, fields, from_db, 0)?;

    write!(file, "\n\n")?;
    Ok(())
}

fn write_object(
    file: &mut File,
    fields: &Fields,
    from_db: bool,
    depth: usize,
) -> anyhow::Result<()> {
    if fields.is_empty() {
        write!(file, "object")?;
    } else {
        writeln!(file, "{{")?;

        let indentation = "\t".repeat(depth);
        for key in fields.keys() {
            write!(file, "{indentation}\t{key}")?;
            write_field(file, &fields[key], from_db, depth)?;
        }

        write!(file, "{indentation}}}")?;
    }

    Ok(())
}

fn write_field(
    file: &mut File,
    field: &FieldTree,
    from_db: bool,
    depth: usize,
) -> anyhow::Result<()> {
    if field.is_optional {
        write!(file, "?")?;
    }

    write!(file, ": ")?;

    if field.is_array {
        write!(file, "Array<")?;
    }

    match &field.r#type {
        FieldType::Node(node) => write_object(file, &node.fields, from_db, depth + 1)?,
        FieldType::Leaf(leaf) => write_primitive(file, &leaf.name, leaf.is_record, from_db)?,
    }

    if field.is_array {
        write!(file, ">")?;
    }

    writeln!(file)?;
    Ok(())
}

fn write_primitive(
    file: &mut File,
    name: &String,
    is_record: bool,
    from_db: bool,
) -> anyhow::Result<()> {
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
    } else if name == "array" || name == "set" {
        // we get here when array or set is used without a generic type parameter
        "[]".to_string()
    } else {
        name.to_string()
    };

    write!(file, "{name}")?;
    Ok(())
}

pub fn create_interface_name(name: &str, from_db: bool) -> String {
    let pascal_case_name = name.to_case(Case::Pascal);

    if from_db {
        format!("Out{pascal_case_name}")
    } else {
        format!("In{pascal_case_name}")
    }
}
