//! # Ui
//!
//! This module contains functions related to printing to terminal I/O. Anything
//! that the user interacts with will be created here.

use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

use crate::folders::{Folder, Style};
use crate::{db, tasks::Task, ToString};

use super::{AppState, Priority};
// use super::{Action, AppState, Priority};

use std::error::Error;
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
/// ```text
/// print_header(AppState::Shop)
/// // Should print:
/// // ===================
/// //   Backlist > Shop
/// // ===================
/// ````
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
    // We loop to retry invalid inputs
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

/// Reads a line of text from stdin after displaying a prompt, trims the input, and returns it.
///
/// # Arguments
///
/// * `prompt: &str` - A string slice that holds the prompt message displayed to the user.
///
/// # Returns
///
/// * `Result<String, io::Error>` which is Ok containing the trimmed string if read successfully, or an Err otherwise.
fn read_trimmed_line(prompt: &str) -> Result<String, io::Error> {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Requests and returns the parent_id from the user.
///
/// # Arguments
///
/// * `conn: &Connection` - A conncetion to the db. `db::read_all_folders()` requires
/// it, so it's required here too.
///
/// # Returns
///
/// * `Result<u32, io::Error>` containing the parent_id if successfully read, or an Err otherwise.
fn request_parent_id(conn: &Connection) -> Result<u32, io::Error> {
    loop {
        let hm = db::read_all_folders(conn, None, "".to_string()).unwrap();

        // Collect HashMap entries into a vector
        let mut entries: Vec<(_, _)> = hm.into_iter().collect();

        // Sort the vector by value alphabetically
        entries.sort_by_key(|entry| entry.1.clone());

        // Print sorted results
        for (i, (_, value)) in entries.iter().enumerate() {
            println!("{}. {}", i + 1, value);
        }

        let selection = read_trimmed_line("\nSelect a folder.\n")?;
        // TODO: Error handling for unwrap()

        match selection.parse::<usize>() {
            Ok(n) => {
                if n >= 1 && n <= entries.len() {
                    let (real_id, _) = entries[n - 1];
                    return Ok(real_id);
                } else {
                    println!("Invalid input!")
                }
            }
            Err(_) => println!("Invalid input!"),
        }
    }
}

/// Requests and returns the task summary from the user.
///
/// # Returns
///
/// * `Result<String, io::Error>` containing the task summary if successfully read, or an Err otherwise.
fn request_task_summary() -> Result<String, io::Error> {
    loop {
        let summary = read_trimmed_line("\nEnter task summary\n")?;

        if !summary.is_empty() {
            return Ok(summary);
        } else {
            println!("The task's summary cannot be empty!")
        }
    }
}

/// Requests an optional description from the user. Returns None if the user enters an empty string.
///
/// # Returns
///
/// * `Result<Option<String>, io::Error>` containing the task description if provided, or None if left blank.
fn request_optional_description() -> Result<Option<String>, io::Error> {
    let description = read_trimmed_line("\nEnter description (or hit <ENTER> to leave blank)\n")?;
    if description.is_empty() {
        Ok(None)
    } else {
        Ok(Some(description))
    }
}

/// Requests the priority of the task from the user and converts it to a `Priority` enum.
///
/// # Returns
///
/// * `Result<Priority, Box<dyn Error>>` which is Ok containing the priority if successfully parsed, or an Err otherwise.
fn request_priority() -> Result<Priority, Box<dyn Error>> {
    loop {
        let input = read_trimmed_line(
            "\nEnter priority\n0. Deprioritized\n1. Default\n2. High Priority\n3. Top Priority\n",
        )?;
        match input.parse::<usize>() {
            Ok(0) => return Ok(Priority::P0),
            Ok(1) => return Ok(Priority::P1),
            Ok(2) => return Ok(Priority::P2),
            Ok(3) => return Ok(Priority::P3),
            Ok(_) | Err(_) => println!("Invalid input!"),
        }
    }
}

/// Requests the type of task from the user, ensuring only valid options (1, 2, or 3) are accepted.
/// Reprompts the user until a valid option is entered.
///
/// # Returns
///
/// * `Result<u32, Box<dyn Error>>` which is Ok containing the task type if successfully parsed, or an Err otherwise.
fn request_task_type() -> Result<u32, Box<dyn Error>> {
    loop {
        let input = read_trimmed_line(
            "\nWhat type of task is this?\n1. One Time\n2. Recurring\n3. Hard Deadline\n",
        )?;
        match input.parse::<u32>() {
            Ok(1) | Ok(2) | Ok(3) => return Ok(input.parse().unwrap()),
            _ => println!("Invalid input!"),
        }
    }
}

/// Requests the interval for recurring tasks from the user, ensuring that only positive integers are accepted.
///
/// # Returns
///
/// * `Result<Option<u32>, Box<dyn Error>>` which is Ok containing the interval in days if a valid input is provided.
fn request_recurring_details() -> Result<Option<u32>, Box<dyn Error>> {
    loop {
        let input = read_trimmed_line("\nHow many days would you like between recurrences?\n")?;
        match input.parse::<u32>() {
            Ok(num) if num > 0 => return Ok(Some(num)),
            _ => println!("Invalid input!"),
        }
    }
}

/// Requests deadline details for tasks with a hard deadline, ensuring that the provided values are valid.
///
/// # Returns
///
/// * `Result<(Option<DateTime<Utc>>, Option<u32>), Box<dyn Error>>` containing the due date and lead days if valid inputs are provided, or None for each if not applicable.
fn request_deadline_details() -> Result<(Option<DateTime<Utc>>, Option<u32>), Box<dyn Error>> {
    let days_until_deadline = loop {
        let input = read_trimmed_line("\nHow many days until the deadline?\n")?;
        match input.parse::<i64>() {
            Ok(num) if num >= 0 => break num, // Ensuring positive value
            _ => println!("Invalid input. Please enter a non-negative number of days."),
        }
    };
    // TODO: This should be set to last midnight + duration
    let due_date = Utc::now() + Duration::days(days_until_deadline);

    let lead_days = loop {
        let input =
            read_trimmed_line("\nHow many days before the deadline would you like to start?\n")?;
        match input.parse::<u32>() {
            Ok(num) if num > 0 => break num, // Ensuring positive value
            _ => println!("Invalid input. Please enter a positive number of days."),
        }
    };

    Ok((Some(due_date), Some(lead_days)))
}

/// Constructs a `Task` object based on user input. Prompts the user for various task details,
/// including summary, description, priority, and type. Depending on the task type, additional
/// information such as recurrence interval or deadline details may also be requested.
///
/// # Arguments
///
/// * `conn: &Connection` - A conncetion to the db. `db::read_all_folders()` requires
/// it, so it's required here too.
///
/// # Returns
///
/// * `Result<Task, Box<dyn Error>>` which is Ok containing the constructed Task object if all inputs are successfully gathered and parsed, or an Err otherwise.
pub fn request_task_input(conn: &Connection) -> Result<Task, Box<dyn Error>> {
    let patent_id = request_parent_id(conn)?;
    let summary = request_task_summary()?;
    let description = request_optional_description()?;
    let priority = request_priority()?;
    let task_type = request_task_type()?;

    let mut repeat_interval: Option<u32> = None;
    let mut due_date: Option<DateTime<Utc>> = None;
    let mut lead_days: Option<u32> = None;

    match task_type {
        2 => repeat_interval = request_recurring_details()?,
        3 => {
            let details = request_deadline_details()?;
            due_date = details.0;
            lead_days = details.1;
        }
        _ => {}
    }

    Ok(Task {
        id: 0, // This value is ignored
        parent_id: patent_id,
        is_archived: false,
        summary,
        description,
        average_duration: None,
        bounty_modifier: 0.0,
        due_date,
        from_date: Utc::now(), // TODO: Set to last midnight
        lead_days,
        priority,
        repeat_interval,
        times_selected: 0,
        times_shown: 0,
    })
}

/// Requests and returns the folder name from the user.
///
/// # Returns
///
/// * `Result<String, io::Error>` containing the folder name if successfully read, or an Err otherwise.
fn request_folder_name() -> Result<String, io::Error> {
    loop {
        let name = read_trimmed_line("\nEnter folder name\n")?;

        if !name.is_empty() {
            return Ok(name);
        } else {
            println!("The folder's name cannot be empty!")
        }
    }
}

/// Requests the style of the folder from the user and converts it to a `Style` enum.
///
/// # Returns
///
/// * `Result<Style, Box<dyn Error>>` which is Ok containing the style if successfully parsed, or an Err otherwise.
fn request_style() -> Result<Style, Box<dyn Error>> {
    loop {
        let input =
            read_trimmed_line("\nEnter folder type\n1. Directory\n2. Selector\n3. Iterator\n")?;
        match input.parse::<usize>() {
            Ok(1) => return Ok(Style::Directory),
            Ok(2) => return Ok(Style::Selector),
            Ok(3) => return Ok(Style::Iterator),
            Ok(_) | Err(_) => println!("Invalid input!"),
        }
    }
}

/// Constructs a `Folder` object based on user input. Prompts the user for various folder details,
/// including TODO.
///
/// # Returns
///
/// * `Result<Folder, Box<dyn Error>>` which is Ok containing the constructed Folder object if all inputs are successfully gathered and parsed, or an Err otherwise.
pub fn request_folder_input(conn: &Connection) -> Result<Folder, Box<dyn Error>> {
    let parent_id = request_parent_id(conn)?;
    let name = request_folder_name()?;
    let style = request_style()?;

    Ok(Folder {
        id: 0,                      // Assuming these values are still hardcoded or otherwise set
        parent_id: Some(parent_id), // TODO: Allow top-level folders to be added
        name: name,
        style: style,
        status: None,
    })
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

/// Displays the funds out to 2 decimal places, includes a line of context.
///
/// # Arguments
///
/// * `funds: f64` - The funds to be displayed.
pub fn display_funds(funds: f64) {
    // Only displays funds to 2 decimal places
    println!("\nYou have ${:.2} remaining", funds);
}

/// Prompts the user to input a transaction amount. Calls `db::add_transaction()`
/// if a vaild input is found.
///
/// # Arguments
///
/// * `conn: &Connection` - A conncetion to the db. `db::add_transaction()` requires
/// it, so it's required here too.
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
