use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    pub id: Option<i64>,
    pub start_time: String,
    pub stop_time: String,
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
    fn test_activity_default() {
        let activity = Activity::default();
        assert_eq!(activity.id, None);
        assert!(activity.start_time.is_empty());
        assert!(activity.stop_time.is_empty());
        assert!(activity.description.is_empty());
        assert!(activity.category_id.is_none());
        assert!(activity.place_id.is_none());
        assert!(activity.tag_names.is_empty());
        assert!(activity.person_names.is_empty());
    }

    #[test]
    fn test_activity_serialization() {
        let activity = Activity {
            id: Some(1),
            start_time: "09:00".to_string(),
            stop_time: "10:00".to_string(),
            description: "Team Meeting".to_string(),
            category_id: Some(1),
            place_id: Some(2),
            category_name: Some("Work".to_string()),
            place_name: Some("Office".to_string()),
            tag_names: vec!["important".to_string()],
            person_names: vec!["Alice".to_string()],
        };

        let serialized = serde_json::to_string(&activity).unwrap();
        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("09:00"));
        assert!(serialized.contains("10:00"));
        assert!(serialized.contains("Team Meeting"));
        // Check renamed fields
        assert!(serialized.contains("\"category\""));
        assert!(serialized.contains("\"place\""));
    }

    #[test]
    fn test_activity_deserialization() {
        let json = json!({
            "id": 2,
            "start_time": "14:00",
            "stop_time": "15:30",
            "description": "Gym Session",
            "category_id": 5,
            "place_id": null,
            "category": "Fitness",
            "place": null,
            "tags": ["health", "weekly"],
            "persons": []
        });

        let activity: Activity = serde_json::from_value(json).unwrap();
        assert_eq!(activity.id, Some(2));
        assert_eq!(activity.start_time, "14:00");
        assert_eq!(activity.stop_time, "15:30");
        assert_eq!(activity.description, "Gym Session");
        assert_eq!(activity.category_id, Some(5));
        assert_eq!(activity.category_name, Some("Fitness".to_string()));
        assert_eq!(activity.tag_names.len(), 2);
    }

    #[test]
    fn test_activity_clone() {
        let activity1 = Activity {
            id: Some(1),
            start_time: "08:00".to_string(),
            stop_time: "09:00".to_string(),
            description: "Morning Run".to_string(),
            category_id: None,
            place_id: None,
            category_name: None,
            place_name: None,
            tag_names: vec![],
            person_names: vec![],
        };

        let activity2 = activity1.clone();
        assert_eq!(activity2.start_time, "08:00");
        assert_eq!(activity2.description, "Morning Run");
    }

    #[test]
    fn test_activity_debug() {
        let activity = Activity::default();
        let debug_str = format!("{:?}", activity);
        assert!(debug_str.contains("Activity"));
        assert!(debug_str.contains("start_time"));
    }
}
