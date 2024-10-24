use std::fs::File;
use std::io::Write;

use convert_case::{Case, Casing};

use crate::{config::Config, Enum, FieldMeta, FieldType, Literal, TableMeta, Union};

#[derive(Debug)]
enum Direction {
    In,
    Out,
}

pub fn write_tables(tables: &Vec<TableMeta>, config: &Config) -> anyhow::Result<()> {
    println!("\nWriting type declaration file...");

    let mut file = File::create(&config.output)?;

    if config.store_in_db {
        write!(&mut file, "{}\n", include_str!("assets/meta_types.ts"))?;
    }

    for table in tables {
        let in_definition = get_table_definition(&table, Direction::In, config);
        let out_definition = get_table_definition(&table, Direction::Out, config);

        write!(file, "{in_definition}\n\n{out_definition}\n\n")?;
    }

    Ok(())
}

fn get_table_definition(table: &TableMeta, direction: Direction, config: &Config) -> String {
    let interface_name = create_interface_name(&table.name, &direction);
    let fields = get_object_definition(&table.fields, &direction, config, true, 1);

    format!("export type {interface_name} = {fields}")
}

fn get_object_definition(
    fields: &Vec<FieldMeta>,
    direction: &Direction,
    config: &Config,
    add_id: bool,
    depth: usize,
) -> String {
    let mut rows = vec!["{".to_string()];

    if add_id {
        let id = match direction {
            Direction::In => "id?: string,".to_string(),
            Direction::Out => "id: string,".to_string(),
        };

        rows.push(format!("{}{id}", indent(depth)));
    }

    for FieldMeta { name, r#type, .. } in fields {
        let optional = match r#type {
            FieldType::Option { .. } => "?",
            _ => "",
        };

        let ts_type = get_ts_type(r#type, direction, config, depth);
        rows.push(format!("{}{name}{optional}: {ts_type},", indent(depth)));
    }

    rows.push(format!("{}}}", indent(depth - 1)));

    rows.join("\n")
}

fn get_ts_type<'a>(
    r#type: &FieldType,
    direction: &Direction,
    config: &Config,
    depth: usize,
) -> String {
    match r#type {
        FieldType::Any => "any".to_string(),
        FieldType::Null => "null".to_string(),
        FieldType::Boolean => "boolean".to_string(),
        FieldType::Number => "number".to_string(),
        FieldType::String => "string".to_string(),
        FieldType::Date => match direction {
            Direction::In => "Date | string".to_string(),
            Direction::Out => "string".to_string(),
        },
        FieldType::Option { inner } => {
            let inner = get_ts_type(inner, direction, config, depth);
            format!("{inner} | undefined")
        }
        FieldType::Object { fields } => match fields {
            Some(fields) => get_object_definition(&fields, direction, config, false, depth + 1),
            None => "object".to_string(),
        },
        FieldType::Record { table } => {
            let record_interface = create_interface_name(&table, direction);

            match direction {
                Direction::In => format!("Required<{record_interface}>['id']"),
                Direction::Out => match config.links_fetched {
                    true => record_interface,
                    false => format!("{record_interface} | {record_interface}['id']"),
                },
            }
        }
        FieldType::Union(union) => match union {
            Union::Normal { variants } => {
                let ts_types: Vec<_> = variants
                    .into_iter()
                    .map(|variant| get_ts_type(variant, direction, config, depth))
                    .collect();

                ts_types.join(" | ")
            }
            Union::Enum(r#enum) => match r#enum {
                Enum::String { variants } => variants
                    .into_iter()
                    .map(|v| format!("'{v}'"))
                    .collect::<Vec<_>>()
                    .join(" | "),
                Enum::Number { variants } => variants
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" | "),
            },
        },
        FieldType::Array { item } => {
            let item_ts_type = get_ts_type(item, direction, config, depth);

            format!("Array<{item_ts_type}>")
        }
        FieldType::Literal(value) => match value {
            Literal::String { value: string } => format!("'{string}'"),
            Literal::Number { value: number } => number.to_string(),
            Literal::Array { items } => {
                let ts_types: Vec<_> = items
                    .into_iter()
                    .map(|kind| get_ts_type(kind, direction, config, depth))
                    .collect();

                format!("[{}]", ts_types.join(", "))
            }
        },
    }
}

fn create_interface_name(name: &str, direction: &Direction) -> String {
    let pascal_case_name = name.to_case(Case::Pascal);

    match direction {
        Direction::In => format!("In{pascal_case_name}"),
        Direction::Out => format!("Out{pascal_case_name}"),
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}
