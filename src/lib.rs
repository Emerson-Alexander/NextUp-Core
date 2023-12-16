mod db;
mod tasks;
mod weighting;

use chrono::{Duration, Utc};

use crate::{
    tasks::{Priority, Task},
    weighting::calculate_weight,
};

// This function is temporary.
// I'm still figuring out how to best use main.rs vs lib.rs.
pub fn go() {
    println!("Starting lib.rs");

    let conn = db::connect_to_db();

    let my_task_1 = Task {
        id: 1,
        is_archived: false,
        summary: String::from("Wash the dishes"),
        description: Some(String::from("Use lots of soap")),
        due_date: Some(Utc::now() + Duration::days(10)),
        from_date: Utc::now(),
        lead_days: Some(30),
        priority: Priority::P3,
        repeat_interval: None,
        times_selected: 5,
        times_shown: 15,
    };

    db::add_task(&conn, my_task_1);
    // db::add_task(&conn, my_task_2);
    // db::add_task(&conn, my_task_3);

    db::delete_task_by_id(&conn, 5);

    let task_list = db::read_all_tasks(&conn);

    for task in task_list {
        let weight = calculate_weight(&task);

        println!("{} - {}", task.summary, weight);
    }

    println!("Finished lib.rs");
}
