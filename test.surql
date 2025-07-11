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
DEFINE FIELD uuid ON schemafull_test TYPE uuid;
DEFINE FIELD float ON schemafull_test TYPE float;
DEFINE FIELD int ON schemafull_test TYPE int;
DEFINE FIELD enum_string ON schemafull_test TYPE "surreal-ts" | "surreal-db";
DEFINE FIELD enum_number ON schemafull_test TYPE 0 | 6;
DEFINE FIELD literal_either ON schemafull_test TYPE "surreal-ts" | 6;
DEFINE FIELD literal_string ON schemafull_test TYPE "surreal-ts";
DEFINE FIELD literal_number ON schemafull_test TYPE 6;
DEFINE FIELD literal_array ON schemafull_test TYPE ["surreal-ts", 6];
DEFINE FIELD literal_object ON schemafull_test TYPE {
  string: string,
  literal: 'literal',
  union: 'literal' | 6,
  object: object,
  array: array<number>,
  literal_object: {
    number: number,
  } 
};
DEFINE FIELD record ON schemafull_test TYPE record<ref_test>;
DEFINE FIELD multi_table_record ON schemafull_test TYPE record<ref_test | ref_test2>;
DEFINE FIELD empty_record ON schemafull_test TYPE record;
DEFINE FIELD array ON schemafull_test TYPE array;
DEFINE FIELD array_array1 ON schemafull_test TYPE array<array>;
DEFINE FIELD record_array ON schemafull_test TYPE array<record<ref_test>>;
DEFINE FIELD optional_item_array ON schemafull_test TYPE array<option<string>>;
DEFINE FIELD either_simple ON schemafull_test TYPE string | bool;
DEFINE FIELD either_complex ON schemafull_test TYPE record<ref_test> | array<record<ref_test>>;
DEFINE FIELD either_array ON schemafull_test TYPE array<string | bool>;
DEFINE FIELD optional_string ON schemafull_test TYPE option<string>;
DEFINE FIELD only_object ON schemafull_test TYPE object;
DEFINE FIELD object ON schemafull_test TYPE object;
DEFINE FIELD object.string ON schemafull_test TYPE string;
DEFINE FIELD object.optional_string ON schemafull_test TYPE option<string>;
DEFINE FIELD object_object ON schemafull_test TYPE object;
DEFINE FIELD object_object.object ON schemafull_test TYPE object;
DEFINE FIELD object_object.object.string ON schemafull_test TYPE string;
DEFINE FIELD array_object1 ON schemafull_test TYPE array<object>;
DEFINE FIELD array_object2 ON schemafull_test TYPE array<object>;
DEFINE FIELD array_object2.*.string ON schemafull_test TYPE string;
DEFINE FIELD array_object_array_string ON schemafull_test TYPE array<object>;
DEFINE FIELD array_object_array_string.*.array ON schemafull_test TYPE array<string>;
DEFINE FIELD array_string1 ON schemafull_test TYPE array<string>;
DEFINE FIELD array_string_max ON schemafull_test TYPE array<string, 3>;
DEFINE FIELD optional_array_string1 ON schemafull_test TYPE option<array<string>>;
DEFINE FIELD set ON schemafull_test TYPE set;
DEFINE FIELD set_object ON schemafull_test TYPE set<object>;
DEFINE FIELD set_object.*.string ON schemafull_test TYPE string;
DEFINE FIELD set_string ON schemafull_test TYPE set<string>;
DEFINE FIELD set_string_max ON schemafull_test TYPE set<string, 3>;
DEFINE FIELD optional_set_string ON schemafull_test TYPE option<set<string>>;
DEFINE FIELD string_with_default ON schemafull_test TYPE string DEFAULT "default";
DEFINE FIELD string_with_value ON schemafull_test TYPE string VALUE $value;
DEFINE FIELD string_with_assert ON schemafull_test TYPE string ASSERT string::is::email($value);
DEFINE FIELD string_with_permission ON schemafull_test TYPE string PERMISSIONS FULL;

DEFINE TABLE ref_test SCHEMAFULL;
DEFINE FIELD name ON ref_test TYPE string;

DEFINE TABLE ref_test2 SCHEMAFULL;
DEFINE FIELD name ON ref_test2 TYPE string;

DEFINE TABLE schemaless SCHEMALESS;

DEFINE TABLE snake_case SCHEMALESS;
DEFINE TABLE UPPER_SNAKE_CASE SCHEMALESS;
DEFINE TABLE camelCase SCHEMALESS;
DEFINE TABLE PascalCase SCHEMALESS;

DEFINE TABLE with_comment SCHEMALESS COMMENT 'table comment';
DEFINE FIELD double_qoute_comment ON with_comment TYPE string COMMENT "double qoute comment";
DEFINE FIELD not_end_comment ON with_comment TYPE string COMMENT "not end comment" DEFAULT "comment";
DEFINE FIELD small_cased_comment ON with_comment TYPE string comment "small cased comment";
DEFINE FIELD no_comment ON with_comment TYPE string;
DEFINE FIELD comment_using_single_qoute ON with_comment TYPE string comment "comment using the ' character";
DEFINE FIELD comment_using_double_qoute ON with_comment TYPE string comment 'comment using the " character';

COMMIT TRANSACTION;