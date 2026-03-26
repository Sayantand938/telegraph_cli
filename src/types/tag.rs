use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tag_default() {
        let tag = Tag::default();
        assert_eq!(tag.id, None);
        assert!(tag.name.is_empty());
    }

    #[test]
    fn test_tag_serialization() {
        let tag = Tag {
            id: Some(1),
            name: "essential".to_string(),
        };
        let serialized = serde_json::to_string(&tag).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("essential"));
    }

    #[test]
    fn test_tag_deserialization() {
        let json = json!({"id": 2, "name": "weekly"});
        let tag: Tag = serde_json::from_value(json).unwrap();
        assert_eq!(tag.id, Some(2));
        assert_eq!(tag.name, "weekly");
    }

    #[test]
    fn test_tag_clone() {
        let tag1 = Tag { id: Some(1), name: "Test".to_string() };
        let tag2 = tag1.clone();
        assert_eq!(tag2.name, "Test");
    }
}
