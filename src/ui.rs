use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

use crate::{db, tasks::Task};

use super::{Action, Priority};

use std::io;

pub fn wait_for_interaction() {
    println!("\nPress <ENTER> to continue\n");

    let mut _input = String::new();
    io::stdin()
        .read_line(&mut _input)
        .expect("Failed to read line");
}

pub fn select_action() -> Option<Action> {
    println!(
        "
===================
  Backlist > Home  
===================

What action would you like to take?

1. See what's up next
2. Add a task
3. Visit the shop\n"
    );

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let selection: Option<u8> = match input.trim().parse() {
        Ok(num) => Some(num),
        Err(_) => None,
    };

    if selection == Some(1) {
        Some(Action::WhatsNext)
    } else if selection == Some(2) {
        Some(Action::AddTask)
    } else if selection == Some(3) {
        Some(Action::Shop)
    } else {
        None
    }
}

pub fn request_task_input() -> Task {
    println!(
        "
=======================
  Backlist > New Task  
=======================

Enter task summary\n"
    );

    let mut summary_io = String::new();

    io::stdin()
        .read_line(&mut summary_io)
        .expect("Failed to read line");

    let summary = summary_io.trim().to_string();

    println!("\nEnter description (or hit <ENTER> to leave blank)\n");

    let mut description_io = String::new();

    io::stdin()
        .read_line(&mut description_io)
        .expect("Failed to read line");

    let mut description: Option<String> = None;

    println!("len is {}", description_io.len());

    if description_io.len() > 1 {
        description = Some(description_io.trim().to_string());
    }

    println!(
        "
Enter priority (or hit <ENTER> to use default)

0. Deprioritized
1. Default
2. High Priority
3. Top Priority\n"
    );

    let mut priority_io = String::new();

    io::stdin()
        .read_line(&mut priority_io)
        .expect("Failed to read line");

    let priority: Priority = match priority_io.trim().parse() {
        Ok(0) => Priority::P0,
        Ok(1) => Priority::P1,
        Ok(2) => Priority::P2,
        Ok(3) => Priority::P3,
        _ => Priority::P1,
    };

    println!(
        "
What type of task is this?

1. One Time
2. Recurring
3. Hard Deadline\n"
    );

    let mut task_type = String::new();

    io::stdin()
        .read_line(&mut task_type)
        .expect("Failed to read line");

    let mut repeat_interval_io = String::new();
    let mut repeat_interval: Option<u32> = None;
    let mut due_date_io = String::new();
    let mut due_date: Option<DateTime<Utc>> = None;
    let mut lead_days_io = String::new();
    let mut lead_days: Option<u32> = None;

    match task_type.trim().parse() {
        Ok(2) => {
            println!("\nHow many days would you like between recurrences?\n");

            io::stdin()
                .read_line(&mut repeat_interval_io)
                .expect("Failed to read line");

            repeat_interval = match repeat_interval_io.trim().parse() {
                Ok(num) => Some(num),
                _ => None,
            };
        }
        Ok(3) => {
            println!("\nHow many days until the deadline?\n");

            io::stdin()
                .read_line(&mut due_date_io)
                .expect("Failed to read line");

            due_date = match due_date_io.trim().parse() {
                Ok(num) => Some(<Utc>::now() + Duration::days(num)),
                _ => None,
            };

            println!("\nHow many days before the deadline would you like to start?\n");

            io::stdin()
                .read_line(&mut lead_days_io)
                .expect("Failed to read line");

            lead_days = match lead_days_io.trim().parse() {
                Ok(num) => Some(num),
                _ => None,
            };
        }
        _ => (),
    };

    Task {
        id: 0,
        is_archived: false,
        summary,
        description,
        due_date,
        from_date: <Utc>::now(),
        lead_days,
        priority,
        repeat_interval,
        times_selected: 0,
        times_shown: 0,
    }
}

pub fn select_task() -> Option<Action> {
    println!("\nSelect a task to complete, or select 0 to edit tasks\n");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let selection: Option<usize> = match input.trim().parse() {
        Ok(num) => Some(num),
        Err(_) => None,
    };

    match selection {
        Some(num) => {
            if num == 0 {
                Some(Action::EditMode)
            } else if num <= 5 {
                Some(Action::SelectTask(num))
            } else {
                None
            }
        }
        None => Some(Action::ReturnToStart),
    }
}

pub fn display_task(task: &Task) {
    println!(
        "
============================
  Backlist > Task Selected
============================

You have selected:

{}",
        task.summary
    );

    if task.description.is_some() {
        println!("    {}", task.description.clone().unwrap());
    }

    println!("\n\n(Debug) ID: {}\n\n", task.id);
}

pub fn display_shop_banner() {
    println!(
        "
===================
  Backlist > Shop
===================
\n"
    );
}

pub fn display_funds(funds: f32) {
    println!("You have ${} remaining", funds);
}

pub fn request_transaction(conn: &Connection) {
    println!("How much would you like to spend?");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let selection: Option<f32> = match input.trim().parse() {
        Ok(num) => Some(num),
        Err(_) => None,
    };

    match selection {
        Some(num) => {
            if num != 0.0 {
                db::add_transaction(conn, num * -1.0)
            }
        }
        None => (),
    }
}
