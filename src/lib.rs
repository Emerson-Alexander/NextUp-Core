mod db;
mod tasks;

use chrono::Utc;

// This function is temporary.
// I'm still figuring out how to best use main.rs vs lib.rs.
pub fn go() {
    println!("Starting lib.rs");

    let conn = db::connect_to_db();

    let my_task_1 = tasks::build_task(
        None,
        true,
        String::from("Do a cool dance"),
        None,
        None,
        Utc::now(),
        None,
        tasks::Priority::P2,
        None,
        0,
        1,
    );
    let my_task_2 = tasks::build_task(
        None,
        true,
        String::from("Wash the dishes"),
        Some(String::from("Use lots of soap")),
        None,
        Utc::now(),
        None,
        tasks::Priority::P2,
        Some(100),
        0,
        1,
    );
    let my_task_3 = tasks::build_task(
        None,
        true,
        String::from("Feed the plants"),
        None,
        None,
        Utc::now(),
        None,
        tasks::Priority::P2,
        None,
        0,
        1,
    );

    db::add_task(&conn, my_task_1);
    db::add_task(&conn, my_task_2);
    db::add_task(&conn, my_task_3);

    db::delete_task_by_id(&conn, 5);

    let task_list = db::read_all_tasks(&conn);

    for task in task_list {
        println!("Here's a task {:?}", task);
    }

    println!("Finished lib.rs");
}
