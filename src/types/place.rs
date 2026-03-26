use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Place {
    pub id: Option<i64>,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_place_default() {
        let place = Place::default();
        assert_eq!(place.id, None);
        assert!(place.name.is_empty());
    }

    #[test]
    fn test_place_serialization() {
        let place = Place {
            id: Some(1),
            name: "Supermarket".to_string(),
        };
        let serialized = serde_json::to_string(&place).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("Supermarket"));
    }

    #[test]
    fn test_place_deserialization() {
        let json = json!({"id": 2, "name": "Office"});
        let place: Place = serde_json::from_value(json).unwrap();
        assert_eq!(place.id, Some(2));
        assert_eq!(place.name, "Office");
    }

    #[test]
    fn test_place_clone() {
        let place1 = Place { id: Some(1), name: "Test".to_string() };
        let place2 = place1.clone();
        assert_eq!(place2.name, "Test");
    }
}
