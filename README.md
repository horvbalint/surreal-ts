# Surreal-ts
Surreal-ts is a simple to use typescript interface genearator for SurrealDB databases.

## Features
- Many supported field types and properties:
  - Optional field
  - Array
  - Record
  - Nested object
  - Datetime
- complex types (eg.: `option<array<record<table>>>`).


## Usage
Surreal-ts is written in Rust, but it is also published on npm, so it can be easily installed/started with: ```npx surreal-ts```

Alternatively one can also clone this repository and build it for themself with ```cargo build```.

**Thank you [@orhun](https://github.com/orhun) for this interesting blog-post on Rust via Npx https://blog.orhun.dev/packaging-rust-for-npm/ !**

## Disclaimer
This project was created as an experiment, and while it works for my usecase it might not work for everyone. I do not take responsibility for problems that might occure due to using this software.

Suggestions and contributions are welcomed :)