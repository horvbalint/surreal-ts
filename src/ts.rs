use std::fs::File;
use std::io::Write;

use crate::{FieldMeta, FieldType, Literal, TableMeta};

#[derive(Debug)]
enum Direction {
    In,
    Out,
}

pub fn write_tables(
    output_path: &str,
    tables: &Vec<TableMeta>,
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

fn get_table_definition(table: &TableMeta, direction: Direction) -> String {
    let interface_name = create_interface_name(&table.name, &direction);
    let fields = get_object_definition(&table.fields, &direction, true, 1);

    format!("export type {interface_name} = {fields}")
}

fn get_object_definition(
    fields: &Vec<FieldMeta>,
    direction: &Direction,
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

        let ts_type = get_ts_type(r#type, direction, depth);
        rows.push(format!("{}{name}{optional}: {ts_type},", indent(depth)));
    }

    rows.push(format!("{}}}", indent(depth - 1)));

    rows.join("\n")
}

fn get_ts_type<'a>(r#type: &FieldType, direction: &Direction, depth: usize) -> String {
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
            let inner = get_ts_type(inner, direction, depth);
            format!("{inner} | undefined")
        }
        FieldType::Object { fields } => match fields {
            Some(fields) => get_object_definition(&fields, direction, false, depth + 1),
            None => "object".to_string(),
        },
        FieldType::Record { table } => {
            let record_interface = create_interface_name(&table, direction);

            match direction {
                Direction::In => format!("Required<{record_interface}>['id']"),
                Direction::Out => format!("{record_interface} | {record_interface}['id']"),
            }
        }
        FieldType::Union { variants } => {
            let ts_types: Vec<_> = variants
                .into_iter()
                .map(|variant| get_ts_type(variant, direction, depth))
                .collect();

            ts_types.join(" | ")
        }
        FieldType::Array { item } => {
            let item_ts_type = get_ts_type(item, direction, depth);

            format!("Array<{item_ts_type}>")
        }
        FieldType::Literal(value) => match value {
            Literal::String { value: string } => format!("'{string}'"),
            Literal::Number { value: number } => number.to_string(),
            Literal::Array { inner: items } => {
                let ts_types: Vec<_> = items
                    .into_iter()
                    .map(|kind| get_ts_type(kind, direction, depth))
                    .collect();

                format!("[{}]", ts_types.join(", "))
            }
        },
    }
}

fn create_interface_name(name: &str, direction: &Direction) -> String {
    match direction {
        Direction::In => format!("In{name}"),
        Direction::Out => format!("Out{name}"),
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}
