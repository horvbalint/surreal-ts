use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use clap::Parser;
use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::{
    bytes::complete::{is_not, take_until1},
    combinator::opt,
    sequence::delimited,
    IResult,
};
use serde::Deserialize;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use itertools::Itertools;

/// A simple typescript definition generator for SurrealDB
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// The connection url to the SurrealDB instance
    #[arg(short, long, default_value_t = String::from("localhost:8000"))]
    connection_url: String,

    /// The root username for the SurrealDB instance
    #[arg(short, long, default_value_t = String::from("root"))]
    username: String,

    /// The root password for the SurrealDB instance
    #[arg(short, long, default_value_t = String::from("root"))]
    password: String,

    /// The namespace to use
    #[arg(short, long)]
    namespace: String,

    /// The database to use
    #[arg(short, long)]
    database: String,

    /// The path where the typescript defintion file will be generated
    #[arg(short, long, default_value_t = String::from("db.d.ts"))]
    output: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let db = Surreal::new::<Ws>(&args.connection_url).await?;

    db.signin(Root {
        username: &args.username,
        password: &args.password,
    })
    .await?;

    db.use_ns(&args.namespace).use_db(&args.database).await?;

    let mut file = File::create(args.output)?;
    
    let mut generator = Generator::new(db)?;
    generator.generate(&mut file).await?;

    Ok(())
}

#[derive(Deserialize, Debug)]
struct DatabaseInfo {
    tables: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct TableInfo {
    fields: HashMap<String, String>,
}

type Fields = HashMap<String, Field>;

struct Generator {
    db: Surreal<Client>,
    tables: HashMap<String, Fields>,
}

impl Generator {
    pub fn new(db: Surreal<Client>) -> anyhow::Result<Self> {
        Ok(Self {
            db,
            tables: HashMap::new(),
        })
    }

    pub async fn generate(&mut self, file: &mut File) -> anyhow::Result<()> {
        println!("\nGenerator warming up...\n");

        let info: Option<DatabaseInfo> = self.db
            .query("INFO FOR DB")
            .await?
            .take(0)?;
            
        let info = info.expect("Failed to get information from the database.");

        for name in info.tables.keys().sorted() {
            self.process_table(file, name).await?
        }

        println!("\nTypes successfully generated ✅");

        Ok(())
    }

    async fn process_table(&mut self, file: &mut File, name: &str) -> anyhow::Result<()> {
        let info: Option<TableInfo> = self.db
            .query(format!("INFO FOR TABLE {name}"))
            .bind(("table", name))
            .await?
            .take(0)?;

        let info = info.expect("Failed to get information from the database.");

        println!("Processing table: {name}");
        self.tables.insert(name.to_string(), HashMap::new());

        for path in info.fields.keys().sorted() {
            Self::process_field(self.tables.get_mut(name).unwrap(), path, &info.fields[path])?;
        }

        self.write_table(file, name, false)?;
        self.write_table(file, name, true)?;

        Ok(())
    }

    fn process_field(tree: &mut Fields, path: &String, definition: &String) -> anyhow::Result<()> {
        let field = Field::from(&definition)?;
        Self::add_to_tree(tree, path.split('.'), field);

        Ok(())
    }

    fn add_to_tree<'a>(fields: &mut Fields, mut steps: impl Iterator<Item=&'a str>, field: Field) {
        let Some(step) = steps.next() else {
            return
        };

        let step = step.split("[").next() // array_key[*] -> array_key
            .expect("Field path contained empty step"); 

        if fields.contains_key(step) {
            let FieldPayload::SubFields(ref mut fields) = fields.get_mut(step).unwrap().payload else {
                unreachable!("Attempted to add field into non-object field");
            };
            Self::add_to_tree(fields, steps, field)
        }
        else {
            fields.insert(step.to_string(), field);
        }
    }

    fn write_table(&mut self, file: &mut File, name: &str, from_db: bool) -> anyhow::Result<()> {
        let interface_name = Self::create_interface_name(name, from_db);
        write!(file, "export type {interface_name} = ")?;

        let fields = self.tables.get_mut(name).unwrap();
        fields.insert("id".to_string(), Field {
            is_optional: !from_db,
            is_array: false,
            payload: FieldPayload::Type {
                name: "string".to_string(),
                is_record: false
            }
        });

        Self::write_object(file, &self.tables[name], from_db, 0)?;

        write!(file, "\n\n")?;
        Ok(())
    }

    fn write_object(file: &mut File, fields: &Fields, from_db: bool, depth: usize) -> anyhow::Result<()> {
        write!(file, "{{\n")?;

        let indentation = "\t".repeat(depth);
        for key in fields.keys().sorted() {
            write!(file, "{indentation}\t{key}")?;
            Self::write_field(file, &fields[key], from_db, depth)?;
        }

        write!(file, "{indentation}}}")?;
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
                Self::write_type(file, name, *is_record, from_db)?;
            },
            FieldPayload::SubFields(fields) => {
                Self::write_object(file, fields, from_db, depth+1)?;
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
            let ref_name = Self::create_interface_name(name, from_db);

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
        } else {
            name.to_string()
        };

        write!(file, "{name}")?;
        Ok(())
    }

    fn create_interface_name(name: &str, from_db: bool) -> String {
        let mut chars = name.chars();
        let capitilzed_name = match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        };
    
        if from_db {
            format!("Out{capitilzed_name}")
        } else {
            format!("In{capitilzed_name}")
        }
    }
}

#[derive(Debug)]
struct Field {
    is_optional: bool,
    is_array: bool,
    payload: FieldPayload,
}

#[derive(Debug)]
enum FieldPayload {
    Type  {
        name: String,
        is_record: bool,
    },
    SubFields(Fields),
}

struct FieldProps<'a> {
    is_optional: bool,
    is_array: bool,
    is_record: bool,
    name: &'a str,
}

impl Field {
    pub fn from(definition: &str) -> anyhow::Result<Self> {
        let (_, raw_type) = Self::parse_type_from_definition(definition)
                .map_err(|err| err.to_owned())?;

        let (_, props) = Self::parse_type(raw_type)
            .map_err(|err| err.to_owned())?;

        let field = Self {
            is_array: props.is_array,
            is_optional: props.is_optional,
            payload: if props.name == "object" {
                FieldPayload::SubFields(HashMap::new())
            } else {
                FieldPayload::Type {
                    name: props.name.to_string(),
                    is_record: props.is_record,
                }
            }
        };

        Ok(field)
    }

    fn parse_type_from_definition(input: &str) -> IResult<&str, &str> {
        let (input, _) = take_until1("TYPE")(input)?;
        let (input, _) = tag("TYPE ")(input)?;
        let (input, raw_type) = is_not(" ")(input)?;

        Ok((input, raw_type))
    }

    fn parse_type(input: &str) -> IResult<&str, FieldProps> {
        let (input, inner) = opt(delimited(tag("option<"), Self::parse_type, tag(">")))(input)?;
        if let Some(props) = inner {
            return Ok((input, FieldProps {is_optional: true, ..props}));
        }

        let (input, inner) = opt(delimited(tag("array<"), Self::parse_type, tag(">")))(input)?;
        if let Some(props) = inner {
            return Ok((input, FieldProps {is_array: true, ..props}));
        }
        
        let (input, inner) = opt(delimited(tag("record<"), is_not(">"), tag(">")))(input)?;
        if let Some(reference) = inner {
            return Ok((input, FieldProps {name: reference, is_record: true, is_array: false, is_optional: false}))
        }

        let (input, name) = alpha1(input)?;
        Ok((input, FieldProps {name, is_record: false, is_array: false, is_optional: false}))
    }
}