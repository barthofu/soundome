use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::string::{string_similarity, SimilarityAlgorithm};

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

    pub fn get_sources(&self) -> Vec<Reference> {
        self.references
            .iter()
            .filter(|r| r.ref_type == ReferenceType::Source)
            .cloned()
            .collect()
    }

    pub fn get_provider(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Provider)
            .cloned()
    }

    pub fn get_providers(&self) -> Vec<Reference> {
        self.references
            .iter()
            .filter(|r| r.ref_type == ReferenceType::Provider)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> Option<Reference> {
        self.references
            .iter()
            .find(|r| r.ref_type == ReferenceType::Metadata)
            .cloned()
    }

    pub fn display(&self) -> String {
        self.name.clone()
    }

    /// Returns a normalized similarity score (between 0 and 1) of the match between two artists
    pub fn compare(&self, other: &Artist) -> f64 {
        let name_similarity = string_similarity(
            &self.name,
            &other.name,
            SimilarityAlgorithm::Smart,
        );

        name_similarity / 100.00
    }
    
    pub fn transpose_metadata(&mut self, other: &Artist) {
        self.name = other.name.clone();
        if let Some(icon) = &other.icon { self.icon = Some(icon.clone()); };

        // only add new references, do not overwrite existing ones
        for ref_item in &other.references {
            let reference_already_exists = self.references
                .iter()
                .any(|r| 
                    r.platform == ref_item.platform && 
                    r.external_id == ref_item.external_id && 
                    r.ref_type == ref_item.ref_type
                );

            if !reference_already_exists {
                self.references.push(ref_item.clone());
            }
        }
    }
}
