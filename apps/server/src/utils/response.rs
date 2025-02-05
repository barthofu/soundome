use rocket::serde::Serialize;
use schemars::JsonSchema;

#[derive(Serialize, JsonSchema)]
pub struct Success {
    pub success: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct Message {
    pub message: String,
}
