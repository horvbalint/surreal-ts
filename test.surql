BEGIN TRANSACTION;

REMOVE NS surreal_ts_test;

DEFINE NS surreal_ts_test;
USE NS surreal_ts_test;

DEFINE DB surreal_ts_test;
USE DB surreal_ts_test;

DEFINE TABLE schemafull_test SCHEMAFULL;
DEFINE FIELD any ON schemafull_test TYPE any;
DEFINE FIELD string ON schemafull_test TYPE string;
DEFINE FIELD bool ON schemafull_test TYPE bool;
DEFINE FIELD datetime ON schemafull_test TYPE datetime;
DEFINE FIELD duration ON schemafull_test TYPE duration;
DEFINE FIELD number ON schemafull_test TYPE number;
DEFINE FIELD decimal ON schemafull_test TYPE decimal;
DEFINE FIELD float ON schemafull_test TYPE float;
DEFINE FIELD int ON schemafull_test TYPE int;
DEFINE FIELD geometry ON schemafull_test TYPE geometry<point>;
DEFINE FIELD record ON schemafull_test TYPE record<ref_test>;
DEFINE FIELD optional_string ON schemafull_test TYPE option<string>;
DEFINE FIELD object ON schemafull_test TYPE object;
DEFINE FIELD object.string ON schemafull_test TYPE string;
DEFINE FIELD object.optional_string ON schemafull_test TYPE option<string>;
DEFINE FIELD flexible_object ON schemafull_test FLEXIBLE TYPE object;
DEFINE FIELD array ON schemafull_test TYPE array;
DEFINE FIELD array_object ON schemafull_test TYPE array<object>;
DEFINE FIELD array_nested_object ON schemafull_test TYPE array<object>;
DEFINE FIELD array_nested_object.string ON schemafull_test TYPE string;
DEFINE FIELD array_string ON schemafull_test TYPE array<string>;
DEFINE FIELD array_string_max ON schemafull_test TYPE array<string, 3>;
DEFINE FIELD optional_array_string ON schemafull_test TYPE option<array<string>>;
DEFINE FIELD set ON schemafull_test TYPE set;
DEFINE FIELD set_object ON schemafull_test TYPE set<object>;
DEFINE FIELD set_nested_object ON schemafull_test TYPE set<object>;
DEFINE FIELD set_nested_object.string ON schemafull_test TYPE string;
DEFINE FIELD set_string ON schemafull_test TYPE set<string>;
DEFINE FIELD set_string_max ON schemafull_test TYPE set<string, 3>;
DEFINE FIELD optional_set_string ON schemafull_test TYPE option<set<string>>;
DEFINE FIELD string_with_default ON schemafull_test TYPE string DEFAULT "default";
DEFINE FIELD string_with_value ON schemafull_test TYPE string VALUE $value;
DEFINE FIELD string_with_assert ON schemafull_test TYPE string ASSERT string::is::email($value);
DEFINE FIELD string_with_assert ON schemafull_test TYPE string PERMISSIONS FULL;

DEFINE TABLE ref_test SCHEMAFULL;
DEFINE FIELD name ON ref_test TYPE string;

DEFINE TABLE schemaless SCHEMALESS;

DEFINE TABLE snake_case SCHEMALESS;
DEFINE TABLE UPPER_SNAKE_CASE SCHEMALESS;
DEFINE TABLE camelCase SCHEMALESS;
DEFINE TABLE PascalCase SCHEMALESS;

COMMIT TRANSACTION;