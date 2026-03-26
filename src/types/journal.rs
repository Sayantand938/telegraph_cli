use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalEntry {
    pub id: Option<i64>,
    pub content: String,
    pub date: Option<String>,
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
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_journal_entry_default() {
        let entry = JournalEntry::default();
        assert_eq!(entry.id, None);
        assert!(entry.content.is_empty());
        assert!(entry.date.is_none());
        assert!(entry.category_id.is_none());
        assert!(entry.place_id.is_none());
        assert!(entry.tag_names.is_empty());
        assert!(entry.person_names.is_empty());
        assert!(entry.created_at.is_empty());
    }

    #[test]
    fn test_journal_entry_serialization() {
        let entry = JournalEntry {
            id: Some(1),
            content: "Dad promised to buy me a Tesla".to_string(),
            date: Some("2026-03-26".to_string()),
            category_id: Some(1),
            place_id: None,
            category_name: Some("Family".to_string()),
            place_name: None,
            tag_names: vec!["promise".to_string()],
            person_names: vec!["Dad".to_string()],
            created_at: "2026-03-26T10:00:00Z".to_string(),
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("Dad promised to buy me a Tesla"));
        assert!(serialized.contains("2026-03-26"));
        assert!(serialized.contains("\"category\""));
        assert!(serialized.contains("\"tags\""));
        assert!(serialized.contains("\"persons\""));
    }

    #[test]
    fn test_journal_entry_deserialization() {
        let json = json!({
            "id": 1,
            "content": "Finished the project",
            "date": "2026-03-25",
            "category_id": 2,
            "place_id": null,
            "category": "Work",
            "place": null,
            "tags": ["milestone", "team"],
            "persons": ["Alice", "Bob"],
            "created_at": "2026-03-25T18:00:00Z"
        });

        let entry: JournalEntry = serde_json::from_value(json).unwrap();
        assert_eq!(entry.id, Some(1));
        assert_eq!(entry.content, "Finished the project");
        assert_eq!(entry.date, Some("2026-03-25".to_string()));
        assert_eq!(entry.category_id, Some(2));
        assert_eq!(entry.category_name, Some("Work".to_string()));
        assert_eq!(entry.tag_names.len(), 2);
        assert_eq!(entry.person_names.len(), 2);
    }

    #[test]
    fn test_journal_entry_minimal() {
        let json = json!({
            "id": 1,
            "content": "Quick note",
            "created_at": "2026-03-26T10:00:00Z"
        });

        let entry: JournalEntry = serde_json::from_value(json).unwrap();
        assert_eq!(entry.id, Some(1));
        assert_eq!(entry.content, "Quick note");
        assert!(entry.date.is_none());
        assert!(entry.category_id.is_none());
        assert!(entry.tag_names.is_empty());
        assert!(entry.person_names.is_empty());
    }

    #[test]
    fn test_journal_entry_clone() {
        let entry1 = JournalEntry {
            id: Some(1),
            content: "Test entry".to_string(),
            date: None,
            category_id: None,
            place_id: None,
            category_name: None,
            place_name: None,
            tag_names: vec![],
            person_names: vec![],
            created_at: "2026-03-26T10:00:00Z".to_string(),
        };

        let entry2 = entry1.clone();
        assert_eq!(entry2.content, "Test entry");
        assert_eq!(entry2.created_at, "2026-03-26T10:00:00Z");
    }

    #[test]
    fn test_journal_entry_debug() {
        let entry = JournalEntry::default();
        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("JournalEntry"));
        assert!(debug_str.contains("content"));
    }
}
