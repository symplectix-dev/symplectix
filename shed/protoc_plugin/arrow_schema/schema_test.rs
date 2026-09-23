//! Tests for the generated Arrow schemas.

use arrow_schema::{
    DataType,
    Field,
    Schema,
    TimeUnit,
};

/// A Parquet group holds at least one column, so `google.protobuf.Empty`
/// has no struct to become. Whether the field was set is the only thing
/// left to record, and a bool records it.
#[test]
fn empty_message_becomes_a_presence_flag() {
    let schema = review_schema::review();
    let marker = schema.field_with_name("marker").unwrap();
    assert_eq!(marker.data_type(), &DataType::Boolean);
}

/// Nullability is proto presence: a scalar without `optional`
/// always has a value, while a message field and a oneof arm
/// can be nullable.
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
    for (name, nullable) in [("id", false), ("product", true), ("marker", true)] {
        assert_eq!(review.field_with_name(name).unwrap().is_nullable(), nullable, "review.{name}");
    }

    // A repeated field is never absent, only empty, and none of its
    // elements is ever null.
    let DataType::List(item) = customer.field_with_name("ids").unwrap().data_type() else {
        panic!("ids is not a list");
    };
    assert!(!item.is_nullable());
}

/// `Any` needs no handling of its own. It is an ordinary message of
/// `type_url` and `value`. There is no shape to build columns from,
/// and the bytes stay readable only by decoding them against `type_url`.
#[test]
fn any_is_just_an_ordinary_message() {
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

/// A wrapper is a message around one scalar, which is how proto3 gave a
/// scalar presence before `optional` existed. Unwrapping it keeps both
/// halves: the scalar's type, and the nullability that being a message
/// field carries. `Product` is reached through an import, where it is a
/// struct rather than a schema of its own.
#[test]
fn a_wrapper_unwraps_to_a_nullable_scalar() {
    let order = order_schema::order();
    let review = review_schema::review();
    let product = product_schema::product();

    let cases: [(&Schema, &str, DataType); 5] = [
        (&order, "quantity", DataType::Int32),
        (&order, "gift", DataType::Boolean),
        (&review, "rating", DataType::Float64),
        (&review, "comment", DataType::Utf8),
        (&product, "image", DataType::Binary),
    ];

    for (schema, name, ty) in cases {
        let field = schema.field_with_name(name).unwrap();
        assert_eq!(field.data_type(), &ty, "{name}");
        assert!(field.is_nullable(), "{name}");
    }
}

/// `optional` gives a plain scalar the presence a wrapper had to be a
/// whole message to carry, and the two reach the same nullable column.
#[test]
fn an_optional_scalar_is_nullable() {
    let order = order_schema::order();
    let coupon = order.field_with_name("coupon").unwrap();
    assert_eq!(coupon.data_type(), &DataType::Utf8);
    assert!(coupon.is_nullable());
}

/// `Struct` reaches itself through `Value`, so it has no finite tree to
/// become, and its values are dynamically typed anyway. Proto defines a
/// JSON mapping for it, and that text is what the column holds.
#[test]
fn struct_becomes_json_text() {
    let product = product_schema::product();
    let attributes = product.field_with_name("attributes").unwrap();
    assert_eq!(attributes.data_type(), &DataType::Utf8);
    assert!(attributes.is_nullable());

    // Text alone would not say the bytes are JSON, so the field carries
    // Arrow's canonical extension for it.
    assert_eq!(attributes.extension_type_name(), Some("arrow.json"));
}
