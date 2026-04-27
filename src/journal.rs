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
    Tournament,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_message_and_event_type() {
        let entry = JournalEntry::new(EventType::Combat, "You attacked a goblin.".to_string());
        assert_eq!(entry.message, "You attacked a goblin.");
        assert!(matches!(entry.event_type, EventType::Combat));
    }

    #[test]
    fn new_sets_timestamp_near_now() {
        let before = Utc::now();
        let entry = JournalEntry::new(EventType::LevelUp, "Level up!".to_string());
        let after = Utc::now();
        assert!(entry.timestamp >= before);
        assert!(entry.timestamp <= after);
    }

    #[test]
    fn new_loot_event_type() {
        let entry = JournalEntry::new(EventType::Loot, "Found a sword.".to_string());
        assert!(matches!(entry.event_type, EventType::Loot));
    }

    #[test]
    fn new_travel_event_type() {
        let entry = JournalEntry::new(EventType::Travel, "Moved to /tmp.".to_string());
        assert!(matches!(entry.event_type, EventType::Travel));
    }

    #[test]
    fn new_death_event_type() {
        let entry = JournalEntry::new(EventType::Death, "You died.".to_string());
        assert!(matches!(entry.event_type, EventType::Death));
    }
}
