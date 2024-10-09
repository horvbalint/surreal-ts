use std::fs::File;
use std::io::Write;
use std::iter;

use bon::builder;
use convert_case::{Case, Casing};
use itertools::Itertools;
use surrealdb::sql::statements::DefineFieldStatement;
use surrealdb::sql::{Kind, Literal};

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
    println!("\nWriting type declaration file...");

    let mut file = File::create(output_path)?;

    if add_table_meta_types {
        write!(&mut file, "{}\n", include_str!("assets/meta_types.ts"))?;
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
    let fields = get_object_definition()
        .fields(&table.fields)
        .direction(direction)
        .add_id(true)
        .call();

    format!("export type {interface_name} = {fields}")
}

#[builder]
fn get_object_definition(
    fields: &Vec<DefineFieldStatement>,
    direction: Direction,
    #[builder(default = false)] add_id: bool,
    #[builder(default)] prefix: String,
    #[builder(default = 1)] depth: usize,
) -> String {
    let mut rows = vec!["{".to_string()];

    if add_id {
        let id = match direction {
            Direction::In => "id?: string,".to_string(),
            Direction::Out => "id: string,".to_string(),
        };

        rows.push(format!("{}{id}", indent(depth)));
    }

    let mut fields = fields.into_iter();
    while let Some(DefineFieldStatement { name, kind, .. }) = &fields.next() {
        let path = name.to_string();
        let name = path[prefix.len()..].to_string();

        let optional = match kind {
            Some(Kind::Option(_)) => "?",
            _ => "",
        };

        let ts_type = get_ts_type(path, kind.clone(), direction, &mut fields, depth);

        rows.push(format!("{}{name}{optional}: {ts_type},", indent(depth)));
    }

    rows.push(format!("{}}}", indent(depth - 1)));

    rows.join("\n")
}

fn get_ts_type<'a>(
    path: String,
    kind: Option<Kind>,
    direction: Direction,
    fields: &mut (impl Iterator<Item = &'a DefineFieldStatement> + std::clone::Clone),
    depth: usize,
) -> String {
    match kind {
        None => "any".to_string(),
        Some(kind) => match kind {
            Kind::Any => "any".to_string(),
            Kind::Null => "null".to_string(),
            Kind::Bool => "boolean".to_string(),
            Kind::Decimal | Kind::Float | Kind::Int | Kind::Number => "number".to_string(),
            Kind::String | Kind::Uuid | Kind::Duration => "string".to_string(),
            Kind::Datetime => match direction {
                Direction::In => "Date | string".to_string(),
                Direction::Out => "string".to_string(),
            },
            Kind::Option(kind) => get_ts_type(path, Some(*kind), direction, fields, depth),
            Kind::Object => {
                let prefix = format!("{path}.");

                let subfields: Vec<_> = fields
                    .take_while_ref(|f| f.name.to_string().starts_with(&prefix))
                    .cloned()
                    .collect();

                if subfields.len() == 0 {
                    "object".to_string()
                } else {
                    get_object_definition()
                        .fields(&subfields)
                        .direction(direction)
                        .prefix(prefix)
                        .depth(depth + 1)
                        .call()
                }
            }
            Kind::Record(vec) => {
                let record_interface = create_interface_name(&vec[0], direction);

                match direction {
                    Direction::In => format!("Required<{record_interface}>['id']"),
                    Direction::Out => format!("{record_interface} | {record_interface}['id']"),
                }
            }
            Kind::Either(kinds) => {
                let ts_types: Vec<_> = kinds
                    .into_iter()
                    .map(|kind| get_ts_type(path.clone(), Some(kind), direction, fields, depth))
                    .collect();

                ts_types.join(" | ")
            }
            Kind::Set(_, _) | Kind::Array(_, _) => {
                let item_ts_type = match fields.next() {
                    Some(item_definition) => {
                        get_ts_type(
                            item_definition.name.to_string(),
                            item_definition.kind.clone(),
                            direction,
                            fields,
                            depth,
                        )
                    },
                    None => "any".to_string()
                };

                format!("Array<{item_ts_type}>")
            }
            Kind::Literal(literal) => match literal {
                Literal::String(string) => string.to_string(),
                Literal::Number(number) => number.to_string(),
                Literal::Array(kinds) => {
                    let ts_types: Vec<_> = kinds
                    .into_iter()
                    .map(|kind| get_ts_type(path.clone(), Some(kind), direction, fields, depth))
                    .collect();

                    format!("[{}]", ts_types.join(", "))
                },
                Literal::Object(map) => {
                    let mut rows = vec!["{".to_string()];

                    for (name, kind) in map {
                        let optional = match kind {
                            Kind::Option(_) => "?",
                            _ => "",
                        };

                        let ts_type = get_ts_type(name.clone(), Some(kind), direction, &mut iter::empty(), depth);

                        rows.push(format!("{}{name}{optional}: {ts_type},", indent(depth)));
                    }

                    rows.push(format!("{}}}", indent(depth - 1)));

                    rows.join("\n")
                },
                _ => unimplemented!(
                    "The type of field '{path}' is not yet supported. Please open an issue on github."
                ),
            },
            _ => unimplemented!(
                "The type of field '{path}' is not yet supported. Please open an issue on github."
            ),
        },
    }
}

fn create_interface_name(name: &str, direction: Direction) -> String {
    let pascal_case_name = name.to_case(Case::Pascal);

    match direction {
        Direction::In => format!("In{pascal_case_name}"),
        Direction::Out => format!("Out{pascal_case_name}"),
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}
