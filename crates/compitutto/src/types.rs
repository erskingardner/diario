use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// A single homework entry
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct HomeworkEntry {
    /// Type of entry (e.g., "compiti", "nota")
    #[serde(rename = "type")]
    pub entry_type: String,

    /// Due date in YYYY-MM-DD format
    pub date: String,

    /// Subject name
    pub subject: String,

    /// Task description
    pub task: String,
}

impl HomeworkEntry {
    pub fn new(entry_type: String, date: String, subject: String, task: String) -> Self {
        Self {
            entry_type,
            date,
            subject,
            task,
        }
    }

    /// Create a deduplication key for this entry
    /// Two entries are considered duplicates if they have the same date, subject, and task
    pub fn dedup_key(&self) -> String {
        format!("{}|{}|{}", self.date, self.subject, self.task)
    }
}

impl Hash for HomeworkEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.date.hash(state);
        self.subject.hash(state);
        self.task.hash(state);
    }
}
