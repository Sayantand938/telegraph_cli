use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_category_default() {
        let cat = Category::default();
        assert_eq!(cat.id, None);
        assert!(cat.name.is_empty());
    }

    #[test]
    fn test_category_serialization() {
        let cat = Category {
            id: Some(1),
            name: "Groceries".to_string(),
        };
        let serialized = serde_json::to_string(&cat).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("Groceries"));
    }

    #[test]
    fn test_category_deserialization() {
        let json = json!({"id": 2, "name": "Dining"});
        let cat: Category = serde_json::from_value(json).unwrap();
        assert_eq!(cat.id, Some(2));
        assert_eq!(cat.name, "Dining");
    }

    #[test]
    fn test_category_clone() {
        let cat1 = Category { id: Some(1), name: "Test".to_string() };
        let cat2 = cat1.clone();
        assert_eq!(cat2.name, "Test");
    }
}
