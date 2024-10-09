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
        let in_definition = get_table_definition(&table, Direction::In);
        let out_definition = get_table_definition(&table, Direction::Out);

        write!(file, "{in_definition}\n\n{out_definition}\n\n")?;
    }

    Ok(())
}

fn get_table_definition(table: &Table, direction: Direction) -> String {
    let interface_name = create_interface_name(&table.table.name, direction);
    let fields = get_object_definition(&table.fields, direction, true);

    format!("export type {interface_name} = {fields}")
}

fn get_object_definition(
    fields: &Vec<DefineFieldStatement>,
    direction: Direction,
    add_id: bool,
) -> String {
    let mut rows = vec!["{".to_string()];

    if add_id {
        match direction {
            Direction::In => rows.push("id?: string,".to_string()),
            Direction::Out => rows.push("id: string,".to_string()),
        }
    }

    let mut fields = fields.iter().peekable();
    while let Some(field) = fields.next() {
        let name = field.name.to_string();
        let (ts_type, optional) =
            get_ts_type(name.clone(), field.kind.clone(), direction, &mut fields);

        rows.push(format!(
            "{name}{}: {ts_type},",
            if optional { "?" } else { "" }
        ));
    }

    rows.push("}".to_string());

    rows.join("\n")
}

fn get_ts_type<'a>(
    path: String,
    kind: Option<Kind>,
    direction: Direction,
    fields: &mut Peekable<impl Iterator<Item = &'a DefineFieldStatement>>,
) -> (String, bool) {
    let optional = match kind {
        Some(Kind::Option(_)) => true,
        _ => false,
    };

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
            Kind::Object => {
                let subfields = fields.take_while(|f| f.name.to_string().starts_with(&path));
            }
            Kind::String => "string".to_string(),
            Kind::Uuid => "string".to_string(),
            Kind::Record(vec) => {
                let record_interface = create_interface_name(&vec[0], direction);

                match direction {
                    Direction::In => format!("{record_interface}['id']"),
                    Direction::Out => format!("{record_interface} | {record_interface}['id']"),
                }
            }
            Kind::Option(kind) => format!(
                "{} | undefined",
                get_ts_type(path, Some(*kind), direction, fields).0
            ),
            Kind::Either(vec) => {
                let ts_types: Vec<_> = vec
                    .into_iter()
                    .map(|kind| get_ts_type(path, Some(kind), direction, fields).0)
                    .collect();

                ts_types.join(" | ")
            }
            Kind::Set(kind, _) => {
                format!(
                    "Array<{}>",
                    get_ts_type(path, Some(*kind), direction, fields).0
                )
            }
            Kind::Array(kind, _) => {
                format!(
                    "Array<{}>",
                    get_ts_type(path, Some(*kind), direction, fields).0
                )
            }
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

    (ts_type, optional)
}

fn create_interface_name(name: &str, direction: Direction) -> String {
    let pascal_case_name = name.to_case(Case::Pascal);

    match direction {
        Direction::In => format!("In{pascal_case_name}"),
        Direction::Out => format!("Out{pascal_case_name}"),
    }
}

// fn get_object_definition(
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
//         FieldType::Node(node) => get_object_definition(file, &node.fields, from_db, depth + 1)?,
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
