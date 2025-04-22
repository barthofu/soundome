use super::Reference;

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: Option<i32>,
    pub name: String,
    pub icon: Option<String>,
    pub references: Vec<Reference>,
}

