use std::fs::File;
use std::io::Write;

use convert_case::{Case, Casing};
use itertools::Itertools;
use surrealdb::sql::statements::DefineFieldStatement;
use surrealdb::sql::Kind;

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
    let fields = get_object_definition(&table.fields, direction, true, "".to_string(), 1);

    format!("export type {interface_name} = {fields}")
}

fn get_object_definition(
    fields: &Vec<DefineFieldStatement>,
    direction: Direction,
    add_id: bool,
    prefix: String,
    depth: usize,
) -> String {
    let mut rows = vec!["{".to_string()];

    if add_id {
        let id = match direction {
            Direction::In => "id?: string,".to_string(),
            Direction::Out => "id: string,".to_string(),
        };

        rows.push(format!("{:indent$}{id}", "", indent = depth * 2));
    }

    let mut fields = fields.into_iter();
    while let Some(field) = fields.next() {
        let path = field.name.to_string();
        let name = path[prefix.len()..].to_string();

        let (ts_type, optional) =
            get_ts_type(path, field.kind.clone(), direction, &mut fields, depth);

        rows.push(format!(
            "{:indent$}{name}{}: {ts_type},",
            "",
            if optional { "?" } else { "" },
            indent = depth * 2
        ));
    }

    rows.push(format!("{:indent$}}}", "", indent = (depth - 1) * 2));

    rows.join("\n")
}

fn get_ts_type<'a>(
    path: String,
    kind: Option<Kind>,
    direction: Direction,
    fields: &mut (impl Iterator<Item = &'a DefineFieldStatement> + std::clone::Clone),
    depth: usize,
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
            Kind::String => "string".to_string(),
            Kind::Uuid => "string".to_string(),
            Kind::Duration => "string".to_string(),
            Kind::Geometry(_vec) => "string".to_string(),
            Kind::Decimal | Kind::Float | Kind::Int | Kind::Number => "number".to_string(),
            Kind::Datetime => match direction {
                Direction::In => "Date | string".to_string(),
                Direction::Out => "string".to_string(),
            },
            Kind::Option(kind) => get_ts_type(path, Some(*kind), direction, fields, depth).0,
            Kind::Object => {
                let prefix = format!("{path}.");

                let subfields = fields
                    .take_while_ref(|f| f.name.to_string().starts_with(&prefix))
                    .cloned()
                    .collect();

                get_object_definition(&subfields, direction, false, prefix, depth + 1)
            }
            Kind::Record(vec) => {
                let record_interface = create_interface_name(&vec[0], direction);

                match direction {
                    Direction::In => format!("{record_interface}['id']"),
                    Direction::Out => format!("{record_interface} | {record_interface}['id']"),
                }
            }
            Kind::Either(vec) => {
                let ts_types: Vec<_> = vec
                    .into_iter()
                    .map(|kind| get_ts_type(path.clone(), Some(kind), direction, fields, depth).0)
                    .collect();

                ts_types.join(" | ")
            }
            Kind::Set(_, _) | Kind::Array(_, _) => {
                let item_definition = fields
                    .next()
                    .expect("No array item definition followed the array definitino");

                format!(
                    "Array<{}>",
                    get_ts_type(
                        item_definition.name.to_string(),
                        item_definition.kind.clone(),
                        direction,
                        fields,
                        depth
                    )
                    .0
                )
            }
            // Kind::Literal(_literal) => todo!(),
            // Kind::Bytes => unimplemented!(),
            // Kind::Point => unimplemented!(),
            // Kind::Function(_vec, _kind) => unimplemented!(),
            // Kind::Range => unimplemented!(),
            _ => unimplemented!("{path}: {kind:#?}"),
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
