use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Reference, ReferenceType};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Artist {
    pub id: Option<i32>,
    pub name: String,
    pub icon: Option<String>,
    pub references: Vec<Reference>,
}

impl Artist {

    pub fn get_source(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Source)
            .cloned()
    }

    pub fn get_provider(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Provider)
            .cloned()
    }

    pub fn display(&self) -> String {
        self.name.clone()
    }
    
    pub fn transpose_metadata(&mut self, other: &Artist) {
        self.name = other.name.clone();
        if let Some(icon) = &other.icon { self.icon = Some(icon.clone()); };
        for ref_item in &other.references {
            if !self.references.iter().any(|r| r.platform == ref_item.platform && r.external_id == ref_item.external_id && r.ref_type == ref_item.ref_type) {
                self.references.push(ref_item.clone());
            }
        }
    }
}
