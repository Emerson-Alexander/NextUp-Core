mod db;
mod finance;
mod tasks;
mod ui;
mod weighting;

use rusqlite::Connection;

use crate::{
    tasks::{Priority, Task},
    weighting::calculate_weight,
};

pub enum Action {
    ReturnToStart,
    AddTask,
    WhatsNext,
    Shop,
    EditMode,
    SelectTask(usize),
}

// This function is temporary.
// I'm still figuring out how to best use main.rs vs lib.rs.
pub fn go() {
    // loop {
    //     match ui::select_action() {
    //         Some(Action::ReturnToStart) => continue,
    //         Some(Action::WhatsNext) => whats_next(),
    //         Some(Action::AddTask) => add_a_task(),
    //         None => continue,
    //     }
    // }

    return;

    // println!("Starting lib.rs");

    // let conn = db::connect_to_db();

    // let my_task_1 = Task {
    //     id: 1,
    //     is_archived: false,
    //     summary: String::from("Wash the dishes"),
    //     description: Some(String::from("Use lots of soap")),
    //     due_date: Some(Utc::now() + Duration::days(10)),
    //     from_date: Utc::now(),
    //     lead_days: Some(30),
    //     priority: Priority::P3,
    //     repeat_interval: None,
    //     times_selected: 5,
    //     times_shown: 15,
    // };

    // db::add_task(&conn, my_task_1);
    // // db::add_task(&conn, my_task_2);
    // // db::add_task(&conn, my_task_3);

    // db::delete_task_by_id(&conn, 5);

    // let task_list = db::read_all_tasks(&conn);

    // for task in task_list {
    //     let weight = calculate_weight(&task);

    //     println!("{} - {}", task.summary, weight);
    // }

    // println!("Finished lib.rs");
}

pub fn startup() {
    println!(
        "
=====================================================================

 /$$$$$$$    Welcome           /$$       /$$ /$$             /$$
| $$__  $$         To         | $$      | $$|__/            | $$
| $$  \\ $$  /$$$$$$   /$$$$$$$| $$   /$$| $$ /$$  /$$$$$$$ /$$$$$$
| $$$$$$$  |____  $$ /$$_____/| $$  /$$/| $$| $$ /$$_____/|_  $$_/
| $$__  $$  /$$$$$$$| $$      | $$$$$$/ | $$| $$|  $$$$$$   | $$
| $$  \\ $$ /$$__  $$| $$      | $$_  $$ | $$| $$ \\____  $$  | $$ /$$
| $$$$$$$/|  $$$$$$$|  $$$$$$$| $$ \\  $$| $$| $$ /$$$$$$$/  |  $$$$/
|_______/  \\_______/ \\_______/|__/  \\__/|__/|__/|_______/    \\___/

====================================================================="
    );

    let conn = db::connect_to_db();

    ui::wait_for_interaction();

    // // TEST BEGINS
    // db::add_transaction(&conn, 100.50);
    // db::add_transaction(&conn, -45.75);
    // // TEST ENDS

    // TEST BEGINS
    // finance::calc_funds(&conn);
    // println!("{:?}", finance::calc_funds(&conn));
    // TEST ENDS

    main_loop(conn)
}

fn main_loop(conn: Connection) {
    loop {
        match ui::select_action() {
            Some(Action::WhatsNext) => whats_next(&conn),
            Some(Action::AddTask) => add_a_task(&conn),
            Some(Action::Shop) => visit_shop(&conn),
            None => continue,
            _ => continue,
        }
    }
}

fn whats_next(conn: &Connection) {
    println!(
        "
====================
Backlist > Up Next  
====================\n"
    );

    let mut task_list = db::read_all_tasks(conn);

    task_list.sort_by(|a, b| {
        calculate_weight(b)
            .partial_cmp(&calculate_weight(a))
            .unwrap()
    });

    if task_list.len() > 5 {
        task_list.drain(5..);
    }

    let mut i = 1;

    for task in task_list.clone() {
        let bounty = finance::adjusted_value(conn, &task);

        println!("{}. ${}\n  - {}", i, bounty, task.summary);

        if task.description.is_some() {
            println!("        {}", task.description.unwrap());
        }

        db::increment_times_shown(conn, task.id, task.times_shown);

        i += 1;
    }

    loop {
        match ui::select_task() {
            Some(Action::ReturnToStart) => break,
            Some(Action::SelectTask(num)) => {
                task_selected(&conn, task_list.get(num - 1).unwrap());
                ui::wait_for_interaction();
                break;
            }
            Some(Action::EditMode) => println!("Coming Soon!"),
            _ => continue,
        }
    }
}

fn add_a_task(conn: &Connection) {
    let task = ui::request_task_input();

    db::add_task(conn, task)
}

fn task_selected(conn: &Connection, task: &Task) {
    ui::display_task(task);
    finance::payout(conn, task);
    db::increment_times_selected(conn, task.id, task.times_selected);

    if task.repeat_interval.is_some() {
        db::reset_from_date(conn, task.id);
    } else {
        db::archive_task(conn, task.id);
    }
}

fn visit_shop(conn: &Connection) {
    ui::display_shop_banner();
    ui::display_funds(finance::calc_funds(conn));
    ui::request_transaction(conn);
    ui::display_funds(finance::calc_funds(conn));
    ui::wait_for_interaction();
}
