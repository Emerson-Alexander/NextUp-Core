use chrono::{DateTime, Utc};
use std::clone::Clone;

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: u32,
    pub is_archived: bool,
    pub summary: String,
    pub description: Option<String>,
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

// // I'm not sure what the point of this function really is. The Rust Book showed
// // making a builder like this, so I did it ¯\_(ツ)_/¯
// pub fn build_task(
//     id: Option<u32>,
//     is_archived: bool,
//     summary: String,
//     description: Option<String>,
//     due_date: Option<DateTime<Utc>>,
//     from_date: DateTime<Utc>,
//     lead_days: Option<u32>,
//     priority: Priority,
//     repeat_interval: Option<u32>,
//     times_selected: u32,
//     times_shown: u32,
// ) -> Task {
//     Task {
//         id: id.unwrap_or(0),
//         is_archived,
//         summary,
//         description,
//         due_date,
//         from_date,
//         lead_days,
//         priority,
//         repeat_interval,
//         times_selected,
//         times_shown,
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_build_task_yes_emptys() {
//         let manual_task = Task {
//             id: 0,
//             is_archived: false,
//             summary: String::from("Wash the dishes"),
//             description: None,
//             due_date: None,
//             from_date: DateTime::from_timestamp(1702423780, 0).unwrap_or_else(|| panic!()),
//             lead_days: None,
//             priority: Priority::P1,
//             repeat_interval: None,
//             times_selected: 0,
//             times_shown: 0,
//         };

//         let constructed_task = build_task(
//             None,
//             false,
//             String::from("Wash the dishes"),
//             None,
//             None,
//             DateTime::from_timestamp(1702423780, 0).unwrap_or_else(|| panic!()),
//             None,
//             Priority::P1,
//             None,
//             0,
//             0,
//         );

//         assert_eq!(manual_task, constructed_task);
//     }

//     #[test]
//     fn test_build_task_no_emptys() {
//         let manual_task = Task {
//             id: 101,
//             is_archived: true,
//             summary: String::from("Wash the dishes"),
//             description: Some(String::from("Use lots of soap")),
//             due_date: Some(DateTime::from_timestamp(1702523780, 0).unwrap_or_else(|| panic!())),
//             from_date: DateTime::from_timestamp(1702423780, 0).unwrap_or_else(|| panic!()),
//             lead_days: Some(5),
//             priority: Priority::P1,
//             repeat_interval: Some(2),
//             times_selected: 555,
//             times_shown: 444,
//         };

//         let constructed_task = build_task(
//             Some(101),
//             true,
//             String::from("Wash the dishes"),
//             Some(String::from("Use lots of soap")),
//             Some(DateTime::from_timestamp(1702523780, 0).unwrap_or_else(|| panic!())),
//             DateTime::from_timestamp(1702423780, 0).unwrap_or_else(|| panic!()),
//             Some(5),
//             Priority::P1,
//             Some(2),
//             555,
//             444,
//         );

//         assert_eq!(manual_task, constructed_task);
//     }
// }
