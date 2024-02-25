//! # Ui
//!
//! This module contains functions related to printing to terminal I/O. Anything
//! that the user interacts with will be created here.

use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

use crate::{db, finance, tasks::Task, ToString};

use super::{AppState, Priority};
// use super::{Action, AppState, Priority};

use std::io;

/// Print the Backlist logo to terminal.
///
/// # Notes
///
/// This function is intentionally untested.
pub fn print_logo() {
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
}

/// Requires the user to press enter before the program will continue.
pub fn wait_for_interaction() {
    println!("\nPress <ENTER> to continue\n");

    let mut _input = String::new();
    io::stdin()
        .read_line(&mut _input)
        .expect("Failed to read line");
}

/// Prints a unified header repersenting the passed AppState
///
/// # Arguments
///
/// * `app_state: AppState` - The AppState to be displayed.
///
/// # Examples
///
/// ```
/// print_header(AppState::Shop)
/// // Should print:
/// // ===================
/// //   Backlist > Shop
/// // ===================
/// ```
///
/// # Notes
///
/// To change what displays before the state title, see `aux_info: String`.
pub fn print_header(app_state: AppState) {
    let title = app_state.to_string();

    let aux_info = String::from("Backlist > ");

    let border_len = title.len() + aux_info.len() + 4;
    let mut border = String::with_capacity(border_len);
    for _ in 0..border_len {
        border.push('=');
    }

    println!(
        "
{}
  {}{}
{}",
        border, aux_info, title, border
    );
}

/// Asks the user to select from a list of AppStates
///
/// # Arguments
///
/// * `states: &[AppState]` - The slice of AppStates for the user to select
/// from. Will display in the order provided.
///
/// # Returns
///
/// The selected AppState
///
/// # Notes
///
/// Will inform the user and retry if the user attempts to select a bad input.
pub fn select_app_state(states: &[AppState]) -> AppState {
    // We loop to retry bad inputs
    loop {
        println!(
            "
What would you like to do?\n"
        );

        // Print the ordered list for the user to select from
        for (index, state) in states.iter().enumerate() {
            println!("{}. {}", index + 1, state.to_string());
        }
        print!("\n");

        // Request user input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Check that the input is valid, then return the AppState or continue the loop.
        match input.trim().parse::<usize>() {
            Ok(num) => {
                // Here we make sure the value selected fits into the array before continuing.
                if num > 0 && num <= states.len() {
                    return states[num - 1].clone();
                } else {
                    println!("\nInvallid Input!");
                    continue;
                }
            }
            Err(_) => {
                println!("\nInvalid Input!");
                continue;
            }
        };
    }
}

pub fn select_task(tasks: &[(Task, f64)]) -> (Task, f64) {
    // We loop to retry bad inputs
    loop {
        println!(
            "
Select a task to complete.\n"
        );

        // Print the ordered list for the user to select from
        for (index, tup) in tasks.iter().enumerate() {
            // Unwrap the tuple
            let (task, bounty) = tup;

            // Display the tasks index, bounty, and summary
            println!("{}. ${}\n  - {}", index + 1, bounty, task.summary);

            // Display the description only if it exists
            if task.description.is_some() {
                println!("        {}", task.description.as_ref().unwrap());
            }
        }
        print!("\n");

        // Request user input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Check that the input is valid, then return the AppState or continue the loop.
        match input.trim().parse::<usize>() {
            Ok(num) => {
                // Here we make sure the value selected fits into the array before continuing.
                if num > 0 && num <= tasks.len() {
                    return tasks[num - 1].clone();
                } else {
                    println!("\nInvallid Input!");
                    continue;
                }
            }
            Err(_) => {
                println!("\nInvalid Input!");
                continue;
            }
        };
    }
}

pub fn request_task_input() -> Task {
    println!("\nEnter task summary\n");

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

// pub fn select_task() -> Option<Action> {
//     println!("\nSelect a task to complete, or select 0 to edit tasks\n");

//     let mut input = String::new();

//     io::stdin()
//         .read_line(&mut input)
//         .expect("Failed to read line");

//     let selection: Option<usize> = match input.trim().parse() {
//         Ok(num) => Some(num),
//         Err(_) => None,
//     };

//     match selection {
//         Some(num) => {
//             if num == 0 {
//                 Some(Action::EditMode)
//             } else if num <= 5 {
//                 Some(Action::SelectTask(num))
//             } else {
//                 None
//             }
//         }
//         None => Some(Action::ReturnToStart),
//     }
// }

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

    println!("\n\n(Debug) ID: {}\n", task.id);
}

// pub fn display_shop_banner() {
//     println!(
//         "
// ===================
//   Backlist > Shop
// ===================
// \n"
//     );
// }

pub fn display_funds(funds: f64) {
    // Only displays funds to 2 decimal places
    println!("\nYou have ${:.2} remaining", funds);
}

pub fn request_transaction(conn: &Connection) {
    println!("\nHow much would you like to spend?");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let selection: Option<f64> = match input.trim().parse() {
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
