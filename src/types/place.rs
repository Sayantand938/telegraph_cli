use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Place {
    pub id: Option<i64>,
    pub name: String,
}
