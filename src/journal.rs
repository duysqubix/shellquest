use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Combat,
    Loot,
    Travel,
    Discovery,
    LevelUp,
    Death,
    Quest,
    Craft,
}

impl JournalEntry {
    pub fn new(event_type: EventType, message: String) -> Self {
        JournalEntry {
            timestamp: Utc::now(),
            event_type,
            message,
        }
    }
}
