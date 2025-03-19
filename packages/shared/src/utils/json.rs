use schemars::{gen::SchemaSettings, SchemaGenerator};
use serde_json::{json, Value};

pub fn generate_json_schema<T: schemars::JsonSchema>(_: T) -> Value {
    let settings = SchemaSettings::default().with(|s| {
        s.inline_subschemas = true;
    });
    let mut generator = SchemaGenerator::new(settings);
    let schema = generator.root_schema_for::<T>();

    json!(schema)
}
