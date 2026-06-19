use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::task;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize)]
#[diesel(table_name = task)]
pub struct TaskEntity {
    pub id: i32,
    pub task_type: String,
    pub status: String,
    pub payload: String,
    pub label: Option<String>,
    pub progress: i32,
    pub total: Option<i32>,
    pub error: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub stats: Option<String>,
}

#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = task)]
pub struct NewTaskEntity {
    pub task_type: String,
    pub status: String,
    pub payload: String,
    pub label: Option<String>,
    pub progress: i32,
    pub total: Option<i32>,
}
