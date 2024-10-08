use std::io::Write;
use std::{fs::File, iter::Peekable};

use convert_case::{Case, Casing};
use surrealdb::sql::Kind;
use surrealdb::sql::{statements::DefineFieldStatement, Field};

use crate::Table;

#[derive(Copy, Clone, Debug)]
enum Direction {
    In,
    Out,
}

pub fn write_tables(
    output_path: &str,
    tables: &Vec<Table>,
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

    for table in tables {
        write_table(&mut file, &table, Direction::In)?;
        write_table(&mut file, &table, Direction::Out)?;
    }

    Ok(())
}

fn write_table(file: &mut File, table: &Table, direction: Direction) -> anyhow::Result<()> {
    let interface_name = create_interface_name(&table.table.name, direction);
    write!(file, "export type {interface_name} = {{")?;

    write!(file, "\nid: string,")?;
    write_fields(file, &mut table.fields.iter().peekable(), direction)?;

    write!(file, "\n}}\n\n")?;
    Ok(())
}

fn write_fields<'a>(
    file: &mut File,
    fields: &mut Peekable<impl Iterator<Item = &'a DefineFieldStatement>>,
    direction: Direction,
) -> anyhow::Result<()> {
    while let Some(field) = fields.next() {
        let name = field.name.to_string();
        let ts_type = get_ts_type(field.kind.clone(), direction)?;

        write!(file, "\n{name}: {ts_type},")?;
    }

    Ok(())
}

fn get_ts_type(kind: Option<Kind>, direction: Direction) -> anyhow::Result<String> {
    let ts_type = match kind {
        None => "any".to_string(),
        Some(kind) => match kind {
            Kind::Any => "any".to_string(),
            Kind::Null => "null".to_string(),
            Kind::Bool => "boolean".to_string(),
            Kind::Datetime => match direction {
                Direction::In => "Date | string".to_string(),
                Direction::Out => "string".to_string(),
            },
            Kind::Decimal => "number".to_string(),
            Kind::Float => "number".to_string(),
            Kind::Int => "number".to_string(),
            Kind::Number => "number".to_string(),
            Kind::Object => todo!(),
            Kind::String => "string".to_string(),
            Kind::Uuid => "string".to_string(),
            Kind::Record(vec) => {
                let record_interface = create_interface_name(&vec[0], direction);

                match direction {
                    Direction::In => format!("{record_interface}['id']"),
                    Direction::Out => format!("{record_interface} | {record_interface}['id']"),
                }
            }
            Kind::Option(kind) => format!("{} | undefined", get_ts_type(Some(*kind), direction)?),
            Kind::Either(vec) => {
                let ts_types: Vec<_> = vec
                    .into_iter()
                    .map(|kind| get_ts_type(Some(kind), direction).unwrap())
                    .collect();

                ts_types.join(" | ")
            }
            Kind::Set(kind, _) => format!("Array<{}>", get_ts_type(Some(*kind), direction)?),
            Kind::Array(kind, _) => format!("Array<{}>", get_ts_type(Some(*kind), direction)?),
            Kind::Literal(_literal) => todo!(),
            Kind::Bytes => unimplemented!(),
            Kind::Duration => unimplemented!(),
            Kind::Point => unimplemented!(),
            Kind::Geometry(_vec) => unimplemented!(),
            Kind::Function(_vec, _kind) => unimplemented!(),
            Kind::Range => unimplemented!(),
            _ => todo!(),
        },
    };

    Ok(ts_type)
}

fn create_interface_name(name: &str, direction: Direction) -> String {
    let pascal_case_name = name.to_case(Case::Pascal);

    match direction {
        Direction::In => format!("In{pascal_case_name}"),
        Direction::Out => format!("Out{pascal_case_name}"),
    }
}

// fn write_object(
//     file: &mut File,
//     fields: &Fields,
//     from_db: bool,
//     depth: usize,
// ) -> anyhow::Result<()> {
//     if fields.is_empty() {
//         write!(file, "object")?;
//     } else {
//         writeln!(file, "{{")?;

//         let indentation = "\t".repeat(depth);
//         for key in fields.keys() {
//             write!(file, "{indentation}\t{key}")?;
//             write_field(file, &fields[key], from_db, depth)?;
//         }

//         write!(file, "{indentation}}}")?;
//     }

//     Ok(())
// }

// fn write_field(
//     file: &mut File,
//     field: &FieldTree,
//     from_db: bool,
//     depth: usize,
// ) -> anyhow::Result<()> {
//     if field.is_optional {
//         write!(file, "?")?;
//     }

//     write!(file, ": ")?;

//     if field.is_array {
//         write!(file, "Array<")?;
//     }

//     match &field.r#type {
//         FieldType::Node(node) => write_object(file, &node.fields, from_db, depth + 1)?,
//         FieldType::Leaf(leaf) => write_primitive(file, &leaf.name, leaf.is_record, from_db)?,
//     }

//     if field.is_array {
//         write!(file, ">")?;
//     }

//     writeln!(file)?;
//     Ok(())
// }

// fn write_primitive(
//     file: &mut File,
//     name: &String,
//     is_record: bool,
//     from_db: bool,
// ) -> anyhow::Result<()> {
//     let name = if is_record {
//         let ref_name = create_interface_name(name, from_db);

//         if from_db {
//             format!("{ref_name}['id'] | {ref_name}")
//         } else {
//             format!("Required<{ref_name}>['id']")
//         }
//     } else if name == "datetime" {
//         if from_db {
//             "string".to_string()
//         } else {
//             "Date | string".to_string()
//         }
//     } else if name == "bool" {
//         "boolean".to_string()
//     } else if name == "decimal" || name == "float" || name == "int" {
//         "number".to_string()
//     } else if name == "duration" || name == "geometry" {
//         "string".to_string()
//     } else if name == "array" || name == "set" {
//         // we get here when array or set is used without a generic type parameter
//         "[]".to_string()
//     } else {
//         name.to_string()
//     };

//     write!(file, "{name}")?;
//     Ok(())
// }
