use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Person {
    pub id: Option<i64>,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_person_default() {
        let person = Person::default();
        assert_eq!(person.id, None);
        assert!(person.name.is_empty());
    }

    #[test]
    fn test_person_serialization() {
        let person = Person {
            id: Some(1),
            name: "John Doe".to_string(),
        };
        let serialized = serde_json::to_string(&person).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("John Doe"));
    }

    #[test]
    fn test_person_deserialization() {
        let json = json!({"id": 2, "name": "Alice"});
        let person: Person = serde_json::from_value(json).unwrap();
        assert_eq!(person.id, Some(2));
        assert_eq!(person.name, "Alice");
    }

    #[test]
    fn test_person_clone() {
        let person1 = Person { id: Some(1), name: "Test".to_string() };
        let person2 = person1.clone();
        assert_eq!(person2.name, "Test");
    }
}
