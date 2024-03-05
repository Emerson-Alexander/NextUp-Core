use chrono::{DateTime, Duration, Utc};
use std::clone::Clone;

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: u32,
    pub parent_id: u32,
    pub is_archived: bool,
    pub summary: String,
    pub description: Option<String>,
    pub average_duration: Option<Duration>,
    pub bounty_modifier: f32,
    pub due_date: Option<DateTime<Utc>>,
    pub from_date: DateTime<Utc>,
    pub lead_days: Option<u32>,
    pub priority: Priority,
    pub repeat_interval: Option<u32>,
    pub times_selected: u32,
    pub times_shown: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
}
