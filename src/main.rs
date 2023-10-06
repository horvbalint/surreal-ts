// A simple to use typescript type definition generator for SurrealDB
// Copyright (C) 2023  Horváth Bálint

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see https://www.gnu.org/licenses/.

use std::collections::BTreeMap;

use clap::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::combinator::rest;
use nom::sequence::pair;
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

mod utils;
mod table_info;
mod ts;
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

    /// Whether to store generated table and field data in the database inside the 'table_info' table
    #[arg(short, long, default_value_t = false)]
    store_in_db: bool,

    /// Whether to generate a typescript definition file describing the tables of the database
    #[arg(short, long, default_value_t = true)]
    generate_ts_file: bool,

    /// The path where the typescript defintion file will be generated
    #[arg(short, long, default_value_t = String::from("db.d.ts"))]
    output: String,
}

type Tables = BTreeMap<String, Table>;
type Fields = BTreeMap<String, Field>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    let mut db = Surreal::new::<Ws>(&args.connection_url).await?;
    db.signin(Root {
        username: &args.username,
        password: &args.password,
    })
    .await?;
    db.use_ns(&args.namespace).use_db(&args.database).await?;

    let mut tables = Generator::process(&mut db).await?;
    print!("\n");
    
    if args.store_in_db {
        table_info::store_tables(&mut db, &mut tables).await?;
    }

    if args.generate_ts_file {
        ts::write_tables(&args.output, &mut tables, args.store_in_db).await?;
    }

    println!("\nAll operations done ✅");

    Ok(())
}

#[derive(Deserialize, Debug)]
struct DatabaseInfo {
    tables: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct TableInfo {
    fields: BTreeMap<String, String>,
}

struct Generator {
    tables: Tables,
}

pub struct Table {
    fields: Fields,
    comment: Option<String>,
}

impl Generator {
    pub async fn process(db: &mut Surreal<Client>) -> anyhow::Result<Tables> {
        let mut generator = Self {
            tables: BTreeMap::new(),
        };

        let info: Option<DatabaseInfo> = db
            .query("INFO FOR DB")
            .await?
            .take(0)?;
        let info = info.expect("Failed to get information of the database.");

        for (name, definition) in info.tables {
            generator.process_table(db, &name, &definition).await?
        }

        Ok(generator.tables)
    }

    async fn process_table(&mut self, db: &mut Surreal<Client>, name: &str, definition: &str) -> anyhow::Result<()> {
        println!("Processing table: {name}");
        
        let info: Option<TableInfo> = db
            .query(format!("INFO FOR TABLE {name}"))
            .bind(("table", name))
            .await?
            .take(0)?;
        let info = info.expect("Failed to get information of the table.");

        let (_, comment) = parse_comment(definition)
            .map_err(|err| err.to_owned())?;

        let table = self.tables.entry(name.to_string()).or_insert(Table {
            fields: BTreeMap::new(),
            comment,
        });

        for path in info.fields.keys() {
            Self::process_field(&mut table.fields, path, &info.fields[path])?;
        }

        Ok(())
    }

    fn process_field(tree: &mut Fields, path: &str, definition: &str) -> anyhow::Result<()> {
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

        if let Some(curr_field) = fields.get_mut(step) {
            let FieldPayload::SubFields(ref mut fields) = curr_field.payload else {
                unreachable!("Attempted to add field into non-object field");
            };
            
            Self::add_to_tree(fields, steps, field)
        }
        else {
            fields.insert(step.to_string(), field);
        }
    }
}

#[derive(Debug)]
pub struct Field {
    is_optional: bool,
    is_array: bool,
    payload: FieldPayload,
    comment: Option<String>,
}

#[derive(Debug)]
pub enum FieldPayload {
    Type  {
        name: String,
        is_record: bool,
    },
    SubFields(Fields),
}

#[derive(Debug)]
pub struct FieldProps<'a> {
    is_optional: bool,
    is_array: bool,
    is_record: bool,
    name: &'a str,
}

impl Field {
    pub fn from(definition: &str) -> anyhow::Result<Self> {
        let (remaining, raw_type) = Self::parse_type_from_definition(definition)
                .map_err(|err| err.to_owned())?;

        let (_, props) = Self::parse_type(raw_type)
            .map_err(|err| err.to_owned())?;

        let (_, comment) = parse_comment(remaining)
            .map_err(|err| err.to_owned())?;

        let field = Self {
            is_array: props.is_array,
            is_optional: props.is_optional,
            comment,
            payload: if props.name == "object" {
                FieldPayload::SubFields(BTreeMap::new())
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
        let (input, raw_type) = alt((take_until1("DEFAULT"), take_until1("VALUE"), take_until1("ASSERT"), take_until1("PERMISSIONS"), take_until1("COMMENT"), rest))(input)?;

        Ok((input, raw_type))
    }

    fn parse_type(input: &str) -> IResult<&str, FieldProps> {
        let (input, inner) = opt(delimited(tag("option<"), Self::parse_type, tag(">")))(input)?;
        if let Some(props) = inner {
            return Ok((input, FieldProps {is_optional: true, ..props}));
        }

        let (input, inner) = opt(delimited(tag("array<"), pair(Self::parse_type, opt(is_not(">"))), tag(">")))(input)?;
        if let Some((props, _)) = inner {
            return Ok((input, FieldProps {is_array: true, ..props}));
        }

        let (input, inner) = opt(delimited(tag("set<"), pair(Self::parse_type, opt(is_not(">"))), tag(">")))(input)?;
        if let Some((props, _)) = inner {
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

fn parse_comment(input: &str) -> IResult<&str, Option<String>> {
    let (input, _) = opt(take_until1("COMMENT"))(input)?;
    let (input, res) = opt(tag("COMMENT "))(input)?;

    if res.is_some() {
        let (input, comment) = delimited(tag("'"), is_not("'"), tag("'"))(input)?;
        Ok((input, Some(comment.to_string())))
    }
    else {
        Ok((input, None))
    }
}