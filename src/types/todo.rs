use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Todo {
    pub id: Option<i64>,
    pub description: String,
    pub status: String,           // pending, in_progress, completed, cancelled
    pub priority: Option<String>, // low, medium, high
    pub due_date: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
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
    fn test_todo_default() {
        let todo = Todo::default();
        assert_eq!(todo.id, None);
        assert!(todo.description.is_empty());
        assert!(todo.status.is_empty());
        assert!(todo.priority.is_none());
        assert!(todo.due_date.is_none());
        assert!(todo.created_at.is_empty());
        assert!(todo.completed_at.is_none());
        assert!(todo.category_id.is_none());
        assert!(todo.place_id.is_none());
        assert!(todo.tag_names.is_empty());
        assert!(todo.person_names.is_empty());
    }

    #[test]
    fn test_todo_serialization() {
        let todo = Todo {
            id: Some(1),
            description: "Buy groceries".to_string(),
            status: "pending".to_string(),
            priority: Some("high".to_string()),
            due_date: Some("2026-03-30".to_string()),
            created_at: "2026-03-26T10:00:00Z".to_string(),
            completed_at: None,
            category_id: Some(1),
            place_id: Some(2),
            category_name: Some("Shopping".to_string()),
            place_name: Some("Supermarket".to_string()),
            tag_names: vec!["urgent".to_string()],
            person_names: vec!["John".to_string()],
        };

        let serialized = serde_json::to_string(&todo).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("Buy groceries"));
        assert!(serialized.contains("pending"));
        assert!(serialized.contains("high"));
        assert!(serialized.contains("2026-03-30"));
        // Check renamed fields
        assert!(serialized.contains("\"category\""));
        assert!(serialized.contains("\"place\""));
        assert!(serialized.contains("\"tags\""));
        assert!(serialized.contains("\"persons\""));
    }

    #[test]
    fn test_todo_deserialization() {
        let json = json!({
            "id": 1,
            "description": "Complete project",
            "status": "in_progress",
            "priority": "medium",
            "due_date": "2026-04-01",
            "created_at": "2026-03-25T09:00:00Z",
            "completed_at": null,
            "category_id": 2,
            "place_id": null,
            "category": "Work",
            "place": null,
            "tags": ["project", "deadline"],
            "persons": ["Alice", "Bob"]
        });

        let todo: Todo = serde_json::from_value(json).unwrap();
        assert_eq!(todo.id, Some(1));
        assert_eq!(todo.description, "Complete project");
        assert_eq!(todo.status, "in_progress");
        assert_eq!(todo.priority, Some("medium".to_string()));
        assert_eq!(todo.due_date, Some("2026-04-01".to_string()));
        assert_eq!(todo.category_id, Some(2));
        assert_eq!(todo.category_name, Some("Work".to_string()));
        assert_eq!(todo.tag_names.len(), 2);
        assert_eq!(todo.person_names.len(), 2);
    }

    #[test]
    fn test_todo_deserialization_minimal() {
        let json = json!({
            "id": 1,
            "description": "Simple task",
            "status": "pending",
            "created_at": "2026-03-26T10:00:00Z"
        });

        let todo: Todo = serde_json::from_value(json).unwrap();
        assert_eq!(todo.id, Some(1));
        assert_eq!(todo.description, "Simple task");
        assert_eq!(todo.status, "pending");
        assert!(todo.priority.is_none());
        assert!(todo.due_date.is_none());
        assert!(todo.completed_at.is_none());
        assert!(todo.tag_names.is_empty());
        assert!(todo.person_names.is_empty());
    }

    #[test]
    fn test_todo_clone() {
        let todo1 = Todo {
            id: Some(1),
            description: "Test task".to_string(),
            status: "pending".to_string(),
            priority: None,
            due_date: None,
            created_at: "2026-03-26T10:00:00Z".to_string(),
            completed_at: None,
            category_id: None,
            place_id: None,
            category_name: None,
            place_name: None,
            tag_names: vec![],
            person_names: vec![],
        };

        let todo2 = todo1.clone();
        assert_eq!(todo2.description, "Test task");
        assert_eq!(todo2.status, "pending");
    }

    #[test]
    fn test_todo_debug() {
        let todo = Todo::default();
        let debug_str = format!("{:?}", todo);
        assert!(debug_str.contains("Todo"));
        assert!(debug_str.contains("description"));
        assert!(debug_str.contains("status"));
    }
}
