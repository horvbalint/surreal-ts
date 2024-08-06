#[cfg(test)]
use super::*;

#[test]
fn basic() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE number;").unwrap(),
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: None,
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn optional() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE option<number>;").unwrap(),
        FieldTree {
            is_optional: true,
            is_array: false,
            comment: None,
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn array() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE array<number>;").unwrap(),
        FieldTree {
            is_optional: false,
            is_array: true,
            comment: None,
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn optional_array() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE option<array<number>>;")
            .unwrap(),
        FieldTree {
            is_optional: true,
            is_array: true,
            comment: None,
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn object() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE object;").unwrap(),
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: None,
            r#type: FieldType::Node(Node {
                fields: BTreeMap::default()
            })
        }
    );
}

#[test]
fn optional_object() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE option<object>;").unwrap(),
        FieldTree {
            is_optional: true,
            is_array: false,
            comment: None,
            r#type: FieldType::Node(Node {
                fields: BTreeMap::default()
            })
        }
    );
}

#[test]
fn array_object() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE array<object>;").unwrap(),
        FieldTree {
            is_optional: false,
            is_array: true,
            comment: None,
            r#type: FieldType::Node(Node {
                fields: BTreeMap::default()
            })
        }
    );
}

#[test]
fn basic_with_comment() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE number COMMENT 'TEST';")
            .unwrap(),
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: Some("TEST".to_string()),
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn optional_with_comment() {
    assert_eq!(
        FieldTree::from(
            "DEFINE FIELD test_field ON test_table TYPE option<number> COMMENT 'TEST';"
        )
        .unwrap(),
        FieldTree {
            is_optional: true,
            is_array: false,
            comment: Some("TEST".to_string()),
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn array_with_comment() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE array<number> COMMENT 'TEST';")
            .unwrap(),
        FieldTree {
            is_optional: false,
            is_array: true,
            comment: Some("TEST".to_string()),
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn optional_array_with_comment() {
    assert_eq!(
        FieldTree::from(
            "DEFINE FIELD test_field ON test_table TYPE option<array<number>> COMMENT 'TEST';"
        )
        .unwrap(),
        FieldTree {
            is_optional: true,
            is_array: true,
            comment: Some("TEST".to_string()),
            r#type: FieldType::Leaf(Leaf {
                name: "number".to_string(),
                is_record: false
            })
        }
    );
}

#[test]
fn object_with_comment() {
    assert_eq!(
        FieldTree::from("DEFINE FIELD test_field ON test_table TYPE object COMMENT 'TEST';")
            .unwrap(),
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: Some("TEST".to_string()),
            r#type: FieldType::Node(Node {
                fields: BTreeMap::default()
            })
        }
    );
}
