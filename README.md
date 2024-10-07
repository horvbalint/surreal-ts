# Surreal-ts
Surreal-ts is a simple to use typescript type definition generator for SurrealDB databases.

## Features
- Store metadata back into the database 
- Many supported field types and properties:
  - **any**
  - **array, array\<T\>, array\<T, number\>**
  - **set, set\<T\>, set\<T, number\>**
  - **bool**
  - **datetime**
  - **decimal**
  - **duration**
  - **float**
  - **int**
  - **number**
  - **object**
  - **option\<T\>**
  - **string**
  - **record**
- nested objects
- complex types (eg.: **option\<array\<record\<table\>\>\>**).

## Compatibility
I try to keep this package up-to-date, because I am using it myself, but I can't always keep-up with the braking changes. If you find that surreal-ts is not working for you, please open an issue.

Latest SurrealDB version tested: `2.0.3`


## Installation
Surreal-ts is a CLI tool written in Rust, but it is also published on npm, so it can be easily installed/started with: ```npx surreal-ts```.

If you want to always get the latest version, use: ```npx surreal-ts@latest``` (recommended).

Alternatively one can also clone this repository and build it for themself with ```cargo build```.

**Thank you [@orhun](https://github.com/orhun) for this interesting blog-post on Rust via npx https://blog.orhun.dev/packaging-rust-for-npm/ !**

## Usage
```
Usage: npx surreal-ts@latest [OPTIONS] -n <NAMESPACE> -d <DATABASE>

Options:
  -c, --connection-url <CONNECTION_URL>
          The connection url to the SurrealDB instance [default: http://localhost:8000]
  -u, --username <USERNAME>
          The root username for the SurrealDB instance [default: root]
  -p, --password <PASSWORD>
          The root password for the SurrealDB instance [default: root]
  -n, --namespace <NAMESPACE>
          The namespace to use
  -d, --database <DATABASE>
          The database to use
  -s, --store-in-db
          Store generated table and field metadata into the database
  -m, --metadata-table-name <METADATA_TABLE_NAME>
          Name of the table to use when the 'store-in-db' flag is enabled [default: table_meta]
      --skip-ts-generation
          Skip the generation of a typescript definition file
  -o, --output <OUTPUT>
          The path where the typescript defintion file will be generated [default: db.d.ts]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Output
The generated file will contain two version for every table found in the database. One of them is prefixed with `In` (eg.: InUser), while the other one is prefixed with `Out` (eg.: OutUser).

In* should be used for every action where you are sending data to the database and Out* should be used for the responses from the database.

All table name will be converted to PascalCase for the type definition names.

## Metadata
If the `--store-in-db` flag is specified, surreal-ts will write the structure of your tables back into the database inside the given table. This metadata can be used for example to automate things (like generating ui for a table).

The generated table will contain a record for every 'normal' table in the database, where each records id is the tables name. The 'user' table, will have an id like: `table_meta:user`. This makes it easy to query the metadata of a specific table.

The stored metadata will have a structure described by these typescript interfaces:

```
export type TableMeta = {
  name: string
  fields: FieldMeta[]
  comment?: string
}

export type FieldMeta = {
  name: string
  isOptional: boolean
  isArray: boolean
  type: string
  comment?: string
  isRecord?: true
  fields?: FieldMeta[]
}
```

These interfaces will also be added to the generated typescript definition file, if the `--store-in-db` flag was specified.

A field will only have an `isRecord` property if it is a record link. In this case, the type property will contain the name of the table it references. Similary the `fields` property will only be present if the field is an 'inline' object. Both of these properties can not be present at the same time.

If a `COMMENT` was provided when defining the table or the fields, its value will be stored in the `comment` property. This can be used to provide additional context for the tables and fields.

## Disclaimer
This project was created as an experiment, and while it works for my usecase it might not work for everyone. I do not take responsibility for problems that might occure due to using this software.

Suggestions and contributions are welcomed :)
