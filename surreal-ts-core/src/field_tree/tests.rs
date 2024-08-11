#[cfg(test)]
use super::*;
#[cfg(test)]
use maplit::btreemap;

#[test]
fn from_basic() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE number;").unwrap(),
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
fn from_optional() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE option<number>;").unwrap(),
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
fn from_array() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE array<number>;").unwrap(),
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
fn from_optional_array() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE option<array<number>>;")
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
fn from_object() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE object;").unwrap(),
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: None,
            r#type: FieldType::Node(Node::default())
        }
    );
}

#[test]
fn from_optional_object() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE option<object>;").unwrap(),
        FieldTree {
            is_optional: true,
            is_array: false,
            comment: None,
            r#type: FieldType::Node(Node::default())
        }
    );
}

#[test]
fn from_array_object() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE array<object>;").unwrap(),
        FieldTree {
            is_optional: false,
            is_array: true,
            comment: None,
            r#type: FieldType::Node(Node::default())
        }
    );
}

#[test]
fn from_basic_with_comment() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE number COMMENT 'TEST';")
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
fn from_optional_with_comment() {
    assert_eq!(
        FieldTree::try_from(
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
fn from_array_with_comment() {
    assert_eq!(
        FieldTree::try_from(
            "DEFINE FIELD test_field ON test_table TYPE array<number> COMMENT 'TEST';"
        )
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
fn from_optional_array_with_comment() {
    assert_eq!(
        FieldTree::try_from(
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
fn from_object_with_comment() {
    assert_eq!(
        FieldTree::try_from("DEFINE FIELD test_field ON test_table TYPE object COMMENT 'TEST';")
            .unwrap(),
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: Some("TEST".to_string()),
            r#type: FieldType::Node(Node::default())
        }
    );
}

#[test]
fn insert_one_step() {
    let mut tree = FieldTree {
        is_optional: false,
        is_array: false,
        comment: None,
        r#type: FieldType::Node(Node::default()),
    };

    let leaf: FieldTree = FieldTree {
        is_optional: false,
        is_array: false,
        comment: None,
        r#type: FieldType::Leaf(Leaf {
            name: "number".to_string(),
            is_record: false,
        }),
    };

    tree.insert("test".to_string(), leaf.clone()).unwrap();
    assert_eq!(
        tree,
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: None,
            r#type: FieldType::Node(Node {
                fields: btreemap! {
                    "test".to_string() => leaf
                }
            })
        }
    );
}

#[test]
fn insert_more_step() {
    let inner_node = FieldTree {
        is_optional: false,
        is_array: false,
        comment: None,
        r#type: FieldType::Node(Node::default()),
    };

    let mut tree = FieldTree {
        is_optional: false,
        is_array: false,
        comment: None,
        r#type: FieldType::Node(Node {
            fields: btreemap! {
                "test".to_string() => inner_node.clone()
            },
        }),
    };

    let leaf = FieldTree {
        is_optional: false,
        is_array: false,
        comment: None,
        r#type: FieldType::Leaf(Leaf {
            name: "number".to_string(),
            is_record: false,
        }),
    };

    tree.insert("test.leaf".to_string(), leaf.clone()).unwrap();

    assert_eq!(
        tree,
        FieldTree {
            is_optional: false,
            is_array: false,
            comment: None,
            r#type: FieldType::Node(Node {
                fields: btreemap! {
                    "test".to_string() => FieldTree {
                        r#type: FieldType::Node(Node {
                            fields: btreemap! {
                                "leaf".to_string() => leaf
                            }
                        }),
                        ..inner_node
                    }
                }
            })
        }
    );
}

#[test]
fn insert_incorrect_path() {
    let mut tree = FieldTree {
        is_optional: false,
        is_array: false,
        comment: None,
        r#type: FieldType::Node(Node::default()),
    };

    let leaf = FieldTree {
        is_optional: false,
        is_array: false,
        comment: None,
        r#type: FieldType::Leaf(Leaf {
            name: "number".to_string(),
            is_record: false,
        }),
    };

    let result = tree.insert("test.leaf".to_string(), leaf);
    assert_eq!(result, Err(FieldTreeError::InsertError));
}
