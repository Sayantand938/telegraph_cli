use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}
