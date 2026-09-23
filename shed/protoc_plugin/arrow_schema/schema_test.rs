//! Tests for the generated Arrow schemas.

use arrow::datatypes::{
    DataType,
    Field,
};

/// The generated schema has to survive a round trip through Arrow's own
/// types, which is what a Parquet writer would do with it.
#[test]
fn customer_schema_matches_the_proto() {
    let schema = customer_schema::customer();

    let names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    assert_eq!(names, ["ids", "type", "payload", "i32", "i64", "reviews", "tallies", "products"]);

    // `repeated Id ids` is a list of structs, and `Id`'s oneof arms are
    // plain nullable siblings inside it.
    let DataType::List(item) = schema.field_with_name("ids").unwrap().data_type() else {
        panic!("ids is not a list");
    };
    let DataType::Struct(arms) = item.data_type() else {
        panic!("ids item is not a struct");
    };
    let arm_names: Vec<&str> = arms.iter().map(|f| f.name().as_str()).collect();
    assert_eq!(arm_names, ["uid", "email", "phone"]);
    assert!(arms.iter().all(|f| f.is_nullable()));
}

#[test]
fn empty_message_becomes_a_presence_flag() {
    let schema = review_schema::review();
    let empty = schema.field_with_name("empty").unwrap();
    assert_eq!(empty.data_type(), &DataType::Boolean);
}

#[test]
fn any_keeps_its_payload_opaque() {
    let schema = customer_schema::customer();
    let DataType::Struct(fields) = schema.field_with_name("payload").unwrap().data_type() else {
        panic!("payload is not a struct");
    };
    let expected: Vec<Field> = vec![
        Field::new("type_url", DataType::Utf8, false),
        Field::new("value", DataType::Binary, false),
    ];
    assert_eq!(fields.iter().map(|f| f.as_ref().clone()).collect::<Vec<_>>(), expected);
}

/// Nullability is proto presence, not a blanket choice: a scalar without
/// `optional` always has a value, while a message field, an `optional`
/// one, and a oneof arm can be absent.
#[test]
fn nullability_follows_proto_presence() {
    let customer = customer_schema::customer();
    for (name, nullable) in [("ids", false), ("type", false), ("i32", false), ("payload", true)] {
        assert_eq!(
            customer.field_with_name(name).unwrap().is_nullable(),
            nullable,
            "customer.{name}"
        );
    }

    let review = review_schema::review();
    for (name, nullable) in [("id", false), ("product", true), ("empty", true)] {
        assert_eq!(review.field_with_name(name).unwrap().is_nullable(), nullable, "review.{name}");
    }

    // A repeated field is never absent, only empty, and none of its
    // elements is ever null.
    let DataType::List(item) = customer.field_with_name("ids").unwrap().data_type() else {
        panic!("ids is not a list");
    };
    assert!(!item.is_nullable());
}

/// A proto map is a repeated entry of key and value, which is what Arrow
/// means by a map. The value follows proto presence like any other
/// field, so a scalar value is not null while a message value is.
#[test]
fn a_proto_map_becomes_an_arrow_map() {
    let schema = customer_schema::customer();

    for (name, value_nullable) in [("tallies", false), ("products", true)] {
        let field = schema.field_with_name(name).unwrap();
        let DataType::Map(entries, sorted) = field.data_type() else {
            panic!("{name} is not a map");
        };
        assert!(!sorted, "{name}");

        // A map is never absent, only empty, and Arrow requires the
        // entries group itself to be non-null.
        assert!(!field.is_nullable(), "{name}");
        assert!(!entries.is_nullable(), "{name}");

        let DataType::Struct(kv) = entries.data_type() else {
            panic!("{name} entries is not a struct");
        };
        let names: Vec<&str> = kv.iter().map(|f| f.name().as_str()).collect();
        assert_eq!(names, ["key", "value"], "{name}");

        assert!(!kv[0].is_nullable(), "{name} key");
        assert_eq!(kv[1].is_nullable(), value_nullable, "{name} value");
    }
}

/// A generated file holds only the messages its own proto declares. A
/// message reached across an import is expanded where it is used, so
/// `Product` is a struct inside `Review` and a schema of its own next
/// door, never a reference between the two files.
#[test]
fn an_imported_message_is_expanded_where_it_is_used() {
    let DataType::Struct(inlined) =
        review_schema::review().field_with_name("product").unwrap().data_type().clone()
    else {
        panic!("product is not a struct");
    };

    let declared = product_schema::product();
    assert_eq!(inlined.as_ref(), declared.fields().as_ref());
}
