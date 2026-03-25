use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
}
