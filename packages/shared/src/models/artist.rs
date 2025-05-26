use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Reference;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Artist {
    pub id: Option<i32>,
    pub name: String,
    pub icon: Option<String>,
    pub references: Vec<Reference>,
}

