use serde_json::{Map, Value};

pub trait JsonSchema {
    fn json_schema() -> Value;
}

impl JsonSchema for i32 {
    fn json_schema() -> Value {
        create_integer_schema()
    }
}

impl JsonSchema for i64 {
    fn json_schema() -> Value {
        create_integer_schema()
    }
}

impl JsonSchema for isize {
    fn json_schema() -> Value {
        create_integer_schema()
    }
}

impl JsonSchema for f64 {
    fn json_schema() -> Value {
        create_number_schema()
    }
}

impl JsonSchema for bool {
    fn json_schema() -> Value {
        create_boolean_schema()
    }
}

impl JsonSchema for String {
    fn json_schema() -> Value {
        create_string_schema()
    }
}

impl<T: JsonSchema> JsonSchema for Option<T> {
    fn json_schema() -> Value {
        create_optional_schema::<T>()
    }
}

impl<T: JsonSchema> JsonSchema for Vec<T> {
    fn json_schema() -> Value {
        create_array_schema::<T>()
    }
}

// Helper functions should be defined at module level
fn create_integer_schema() -> Value {
    let mut schema = Map::new();
    schema.insert("type".to_string(), Value::String("integer".to_string()));
    Value::Object(schema)
}

fn create_number_schema() -> Value {
    let mut schema = Map::new();
    schema.insert("type".to_string(), Value::String("number".to_string()));
    Value::Object(schema)
}

fn create_boolean_schema() -> Value {
    let mut schema = Map::new();
    schema.insert("type".to_string(), Value::String("boolean".to_string()));
    Value::Object(schema)
}

fn create_string_schema() -> Value {
    let mut schema = Map::new();
    schema.insert("type".to_string(), Value::String("string".to_string()));
    Value::Object(schema)
}

fn create_optional_schema<T: JsonSchema>() -> Value {
    let mut schema = Map::new();
    let mut one_of = Vec::new();

    let mut null_schema = Map::new();
    null_schema.insert("type".to_string(), Value::String("null".to_string()));
    one_of.push(Value::Object(null_schema));

    one_of.push(T::json_schema());

    schema.insert("oneOf".to_string(), Value::Array(one_of));
    Value::Object(schema)
}

fn create_array_schema<T: JsonSchema>() -> Value {
    let mut schema = Map::new();
    schema.insert("type".to_string(), Value::String("array".to_string()));
    schema.insert("items".to_string(), T::json_schema());
    Value::Object(schema)
}
