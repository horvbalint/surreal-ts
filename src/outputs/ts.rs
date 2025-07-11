use std::fs::File;
use std::io::Write;

use convert_case::{Case, Casing};
use itertools::Itertools;

use crate::{Enum, FieldMetas, FieldType, Literal, TableMeta, TableMetas, Union, config::Config};

#[derive(Debug, PartialEq)]
enum Direction {
    In,
    Out,
}

pub struct TSGenerator<'a> {
    config: &'a Config,
}

impl<'a> TSGenerator<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn write_tables(&self, tables: &TableMetas) -> anyhow::Result<()> {
        println!("\nWriting type declaration file...");

        let mut file = File::create(&self.config.output)?;

        if self.config.target_sdk {
            writeln!(
                &mut file,
                "import {{ RecordId, Duration, Uuid, Decimal }} from \"surrealdb\";\n"
            )?;
        }

        writeln!(file, "// ---------- TABLE TYPES ----------")?;
        for (name, meta) in tables {
            let in_definition = self.get_table_definition(&name, &meta, Direction::In);
            let out_definition = self.get_table_definition(&name, &meta, Direction::Out);

            write!(file, "{in_definition}\n\n{out_definition}\n\n")?;
        }

        if !self.config.no_meta {
            writeln!(file, "// ---------- TABLE META STRUCTURE ----------")?;
            let content = serde_json::to_string_pretty(tables)?;
            write!(
                file,
                "export const tables = {content} as const satisfies Record<string, TableMeta>\n\n"
            )?;
        }

        if self.config.store_meta_in_db || !self.config.no_meta {
            writeln!(file, "// ---------- TABLE META TYPES ----------")?;
            write!(&mut file, "{}\n", include_str!("../assets/meta_types.ts"))?;
        }

        Ok(())
    }

    fn get_table_definition(&self, name: &str, meta: &TableMeta, direction: Direction) -> String {
        let interface_name = create_interface_name(name, &direction);
        let fields = self.get_object_definition(&meta.fields, &direction, Some(name), 1);

        format!("export type {interface_name} = {fields}")
    }

    fn get_object_definition(
        &self,
        fields: &FieldMetas,
        direction: &Direction,
        table_name: Option<&str>,
        depth: usize,
    ) -> String {
        let mut rows = vec!["{".to_string()];

        if let Some(table_name) = table_name {
            let record_type = match self.config.target_sdk {
                true => format!("RecordId<\"{table_name}\">"),
                false => "string".to_string(),
            };

            let id = match direction {
                Direction::In => format!("id?: {record_type},"),
                Direction::Out => format!("id: {record_type},"),
            };

            rows.push(format!("{}{id}", indent(depth)));
        }

        for (name, meta) in fields {
            let optional = matches!(meta.r#type, FieldType::Option { .. })
                || (*direction == Direction::In && meta.has_default);

            let optional = if optional { "?" } else { "" };

            let ts_type = self.get_ts_type(&meta.r#type, direction, depth);
            rows.push(format!("{}{name}{optional}: {ts_type},", indent(depth)));
        }

        rows.push(format!("{}}}", indent(depth - 1)));

        rows.join("\n")
    }

    fn get_ts_type(&self, r#type: &FieldType, direction: &Direction, depth: usize) -> String {
        match r#type {
            FieldType::Any => "any".to_string(),
            FieldType::Null => "null".to_string(),
            FieldType::Boolean => "boolean".to_string(),
            FieldType::Number => "number".to_string(),
            FieldType::String => "string".to_string(),
            FieldType::Bytes => "ArrayBuffer".to_string(),
            FieldType::Decimal => match self.config.target_sdk {
                true => "Decimal".to_string(),
                false => "number".to_string(),
            },
            FieldType::Duration => match self.config.target_sdk {
                true => "Duration".to_string(),
                false => "string".to_string(),
            },
            FieldType::Uuid => match self.config.target_sdk {
                true => "Uuid".to_string(),
                false => "string".to_string(),
            },
            FieldType::Date => match direction {
                Direction::In => "Date | string".to_string(),
                Direction::Out => "string".to_string(),
            },
            FieldType::Option { inner } => {
                let inner = self.get_ts_type(inner, direction, depth);
                format!("{inner} | undefined")
            }
            FieldType::Object { fields } => match fields {
                Some(fields) => self.get_object_definition(&fields, direction, None, depth + 1),
                None => "object".to_string(),
            },
            FieldType::Record { tables } => {
                if tables.is_empty() {
                    let record_type = match self.config.target_sdk {
                        true => "RecordId",
                        false => "string",
                    };

                    match direction {
                        Direction::In => record_type.to_string(),
                        Direction::Out => match self.config.links_fetched {
                            true => "unknown".to_string(),
                            false => format!("unknown | {record_type}"),
                        },
                    }
                } else {
                    tables
                        .iter()
                        .map(|table| {
                            let record_interface = create_interface_name(&table, direction);

                            match direction {
                                Direction::In => format!("Required<{record_interface}>['id']"),
                                Direction::Out => match self.config.links_fetched {
                                    true => record_interface,
                                    false => {
                                        format!("{record_interface} | {record_interface}['id']")
                                    }
                                },
                            }
                        })
                        .join(" | ")
                }
            }
            FieldType::Union(union) => match union {
                Union::Normal { variants } => {
                    let ts_types: Vec<_> = variants
                        .into_iter()
                        .map(|variant| self.get_ts_type(variant, direction, depth))
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
                let item_ts_type = self.get_ts_type(item, direction, depth);

                format!("Array<{item_ts_type}>")
            }
            FieldType::Literal(value) => match value {
                Literal::String { value: string } => format!("'{string}'"),
                Literal::Number { value: number } => number.to_string(),
                Literal::Array { items } => {
                    let ts_types: Vec<_> = items
                        .into_iter()
                        .map(|kind| self.get_ts_type(kind, direction, depth))
                        .collect();

                    format!("[{}]", ts_types.join(", "))
                }
            },
        }
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
