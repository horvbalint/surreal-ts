# Surreal-ts

Surreal-ts is a simple to use typescript type definition and table structure generator for SurrealDB.

## Features

- Supports most of the SurrealDB field types
  - all primitives
  - complex types (eg.: array, option, object)
  - simple literals (eg.: 'foo' | 'bar')
  - complex literals (eg.: object literals)
  - arbitrary nesting
- Generate an object describing the structure of the tables and other metadata
  - append this structure to the output typescript file
  - save this structure inside the database

## Compatibility

I try to keep this package up-to-date, because I am using it myself, but I can't always keep-up with the braking changes. If you find that surreal-ts is not working for you, please open an issue.

Latest SurrealDB version tested: `2.0.4`

## Installation

Surreal-ts is a CLI tool written in Rust, but it is published on npm, so it can be easily installed/started with: `npx surreal-ts`.

If you want to always get the latest version, use: `npx surreal-ts@latest` (recommended).

Alternatively one can also clone this repository and build it for themself with `cargo build`.

**Thank you [@orhun](https://github.com/orhun) for this interesting blog-post on Rust via npx https://blog.orhun.dev/packaging-rust-for-npm/ !**

## Usage

```
Usage: npx surreal-ts@latest -n <NAMESPACE> -d <DATABASE> [OPTIONS]

Options:
  -a, --address <ADDRESS>
          The address to the SurrealDB instance [default: http://localhost:8000]
  -u, --username <USERNAME>
          The root username for the SurrealDB instance [default: root]
  -p, --password <PASSWORD>
          The root password for the SurrealDB instance [default: root]
  -n, --namespace <NAMESPACE>
          The namespace to use
  -d, --database <DATABASE>
          The database to use
  -l, --links-fetched
          Treat record types as FETCHED version of the linked table
  -s, --store-meta-in-db
          Store generated table and field metadata into the database
  -t, --metadata-table-name <METADATA_TABLE_NAME>
          Name of the table to use when the 'store-in-db' flag is enabled [default: table_meta]
      --no-meta
          Skip adding the table meta descriptors to the output ts file
      --skip-ts-generation
          Skip the generation of the typescript definition file
  -o, --output <OUTPUT>
          Path where the typescript defintion file will be generated [default: db.ts]
  -c, --config-file-path <CONFIG_FILE_PATH>
          Path to the configuration JSON file
  -h, --help
          Print help
  -V, --version
          Print version
```

### File configuration

Since Surreal-ts supports many configuration options and projects usually use the same options every time, it is possible to provide all the options in the form of a `json` file. The keys of the json are the long names of the cli options.

## Output

The generated file can contain three sections:

### Table type definitions

This section contains two version for every table found in the database. One of them is prefixed with `In` (eg.: InUser), while the other one is prefixed with `Out` (eg.: OutUser).

`In*` should be used for every action where you are sending data to the database and `Out*` should be used for the responses from the database.

All table name will be converted to PascalCase for the type definition names.

### Table structures and metadata

This section contains an exported typescript object describing every table and their fields. This object can be used to get the possible values of a literal field or to generate ui elements based on the database structure.

If the `store-meta-in-db` options is true, surreal-ts will write this object back into the database inside the table specified in option `metadata-table-name`.

The generated table will contain a record for every table in the database, where each records id is the tables name. The 'user' table, will have an id like: `table_meta:user`. This makes it easy to query the structure and metadata of a specific table.

The form of this object is described by the content of the following section.

### Table structure type definitions

This section contains type definitions for the table structure object and its parts:

```ts
export type Tables = Record<string, TableMeta>;
export type Fields = Record<string, FieldMeta>;

export type TableMeta = {
  fields: Record<string, FieldMeta>;
  comment?: string;
};

export type TableMetaFromDb = TableMeta & {
  id: string;
};

export type FieldMeta = {
  comment?: string;
  type: FieldType;
  hasDefault?: true;
};

export type FieldType =
  | FieldTypes.Simple
  | FieldTypes.Option
  | FieldTypes.Object
  | FieldTypes.Record
  | FieldTypes.Array
  | FieldTypes.Union
  | FieldTypes.StringEnumUnion
  | FieldTypes.NumberEnumUnion
  | FieldTypes.LiteralNumber
  | FieldTypes.LiteralString
  | FieldTypes.LiteralArray;

export namespace FieldTypes {
  export type Simple = {
    name: "any" | "null" | "boolean" | "string" | "number" | "date" | "bytes";
  };

  export type Option = {
    name: "option";
    inner: FieldType;
  };

  export type Object = {
    name: "object";
    fields: null | Fields;
  };

  export type Record = {
    name: "record";
    table: string;
  };

  export type Array = {
    name: "array";
    item: FieldType;
  };

  export type Union = {
    name: "union";
    variants: FieldType[];
  };

  export type StringEnumUnion = {
    name: "union";
    enum: "string";
    variants: string[];
  };

  export type NumberEnumUnion = {
    name: "union";
    enum: "number";
    variants: number[];
  };

  export type LiteralNumber = {
    name: "literal";
    kind: "number";
    value: number;
  };

  export type LiteralString = {
    name: "literal";
    kind: "string";
    value: string;
  };

  export type LiteralArray = {
    name: "literal";
    kind: "array";
    items: FieldType[];
  };
}
```

## Disclaimer

This project was created as an experiment, and while it works for my usecase it might not work for everyone. I do not take responsibility for problems that might occure due to using this software.

Suggestions and contributions are welcomed :)
