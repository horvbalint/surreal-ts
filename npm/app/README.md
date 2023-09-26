# Surreal-ts
Surreal-ts is a simple to use typescript type definition generator for SurrealDB databases.

## Features
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
          The connection url to the SurrealDB instance [default: localhost:8000]
  -u, --username <USERNAME>
          The root username for the SurrealDB instance [default: root]
  -p, --password <PASSWORD>
          The root password for the SurrealDB instance [default: root]
  -n, --namespace <NAMESPACE>
          The namespace to use
  -d, --database <DATABASE>
          The database to use
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

## Disclaimer
This project was created as an experiment, and while it works for my usecase it might not work for everyone. I do not take responsibility for problems that might occure due to using this software.

Suggestions and contributions are welcomed :)