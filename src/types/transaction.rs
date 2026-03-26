use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub id: Option<i64>,
    pub amount: f64,
    pub kind: String,
    pub description: String,
    pub category_id: Option<i64>,
    pub place_id: Option<i64>,
    #[serde(rename = "category", default)]
    pub category_name: Option<String>,
    #[serde(rename = "place", default)]
    pub place_name: Option<String>,
    #[serde(rename = "tags", default)]
    pub tag_names: Vec<String>,
    #[serde(rename = "persons", default)]
    pub person_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transaction_default() {
        let tx = Transaction::default();
        assert_eq!(tx.id, None);
        assert_eq!(tx.amount, 0.0);
        assert!(tx.kind.is_empty());
        assert!(tx.description.is_empty());
        assert!(tx.category_id.is_none());
        assert!(tx.place_id.is_none());
        assert!(tx.category_name.is_none());
        assert!(tx.place_name.is_none());
        assert!(tx.tag_names.is_empty());
        assert!(tx.person_names.is_empty());
    }

    #[test]
    fn test_transaction_serialization() {
        let tx = Transaction {
            id: Some(1),
            amount: 99.99,
            kind: "shopping".to_string(),
            description: "Groceries".to_string(),
            category_id: Some(1),
            place_id: Some(2),
            category_name: Some("Food".to_string()),
            place_name: Some("Supermarket".to_string()),
            tag_names: vec!["essential".to_string()],
            person_names: vec!["John".to_string()],
        };

        let serialized = serde_json::to_string(&tx).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("99.99"));
        assert!(serialized.contains("shopping"));
        assert!(serialized.contains("Groceries"));
        // Check renamed fields
        assert!(serialized.contains("\"category\""));
        assert!(serialized.contains("\"place\""));
        assert!(serialized.contains("Food"));
        assert!(serialized.contains("Supermarket"));
    }

    #[test]
    fn test_transaction_deserialization() {
        let json = json!({
            "id": 1,
            "amount": 50.0,
            "kind": "entertainment",
            "description": "Movies",
            "category_id": 4,
            "place_id": null,
            "category": "Fun",
            "place": null,
            "tags": ["weekend"],
            "persons": ["Alice", "Bob"]
        });

        let tx: Transaction = serde_json::from_value(json).unwrap();
        assert_eq!(tx.id, Some(1));
        assert_eq!(tx.amount, 50.0);
        assert_eq!(tx.kind, "entertainment");
        assert_eq!(tx.description, "Movies");
        assert_eq!(tx.category_id, Some(4));
        assert_eq!(tx.category_name, Some("Fun".to_string()));
        assert_eq!(tx.tag_names, vec!["weekend".to_string()]);
        assert_eq!(tx.person_names, vec!["Alice".to_string(), "Bob".to_string()]);
    }

    #[test]
    fn test_transaction_deserialization_with_nulls() {
        let json = json!({
            "id": null,
            "amount": 25.0,
            "kind": "test",
            "description": "Test transaction",
            "category_id": null,
            "place_id": null,
            "category": null,
            "place": null,
            "tags": [],
            "persons": []
        });

        let tx: Transaction = serde_json::from_value(json).unwrap();
        assert_eq!(tx.id, None);
        assert_eq!(tx.amount, 25.0);
        assert!(tx.category_id.is_none());
        assert!(tx.tag_names.is_empty());
        assert!(tx.person_names.is_empty());
    }

    #[test]
    fn test_transaction_clone() {
        let tx1 = Transaction {
            id: Some(1),
            amount: 100.0,
            kind: "test".to_string(),
            description: "Test".to_string(),
            category_id: None,
            place_id: None,
            category_name: None,
            place_name: None,
            tag_names: vec![],
            person_names: vec![],
        };

        let tx2 = tx1.clone();
        assert_eq!(tx2.id, Some(1));
        assert_eq!(tx2.amount, 100.0);
    }

    #[test]
    fn test_transaction_debug() {
        let tx = Transaction::default();
        let debug_str = format!("{:?}", tx);
        assert!(debug_str.contains("Transaction"));
        assert!(debug_str.contains("amount"));
    }
}
