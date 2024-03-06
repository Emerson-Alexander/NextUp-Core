mod db;
mod finance;
mod folders;
mod tasks;
mod ui;
mod weighting;

use rusqlite::Connection;

use crate::{
    tasks::{Priority, Task},
    weighting::calculate_weight,
};

/// Enumerates the possible states that the application can be in.
#[derive(Clone)]
enum AppState {
    /// Walks the user through adding a new folder to the folders table.
    AddFolder,
    /// Walks the user through adding a new task to the tasks table.
    AddTask,
    /// Allows the user to edit a specific task.
    _EditTask,
    /// Loops AppState::SelectAppState(). May add more functionality later.
    MainLoop,
    /// Where user can make adjustments to their funds.
    Shop,
    /// Presents the user with 5 possible tasks to select.
    ToDo,
}

trait ToString {
    fn to_string(&self) -> &'static str;
}

impl ToString for AppState {
    fn to_string(&self) -> &'static str {
        match self {
            AppState::AddFolder => "Add Folder",
            AppState::AddTask => "Add Task",
            AppState::_EditTask => "Edit Task",
            AppState::MainLoop => "Home",
            AppState::Shop => "Shop",
            AppState::ToDo => "ToDo",
        }
    }
}

/// Assumes the application state specified
///
/// # Arguments
///
/// * `state: AppState` - Determines which state to assume.
/// * `conn: Option<&Connection>` - Allows the new state to connect to the db
/// if necessary.
fn assume_state(state: AppState, conn: Option<&Connection>) {
    // Writing this once to avoid repeating myself
    let db_lost =
        String::from("Value was None, but expected Some(&Connection).\nLost connection to db.");

    match state {
        AppState::AddFolder => add_folder(conn.expect(&db_lost)),
        AppState::AddTask => add_task(conn.expect(&db_lost)),
        AppState::_EditTask => unimplemented!(),
        AppState::MainLoop => main_loop(conn.expect(&db_lost)),
        AppState::Shop => shop(conn.expect(&db_lost)),
        AppState::ToDo => to_do(conn.expect(&db_lost)),
    }
}

/// Initializes the program for use by a user through the TUI.
///
/// # Notes
///
/// This function is intentionally untested.
pub fn startup() {
    ui::print_logo();

    let conn = db::connect_to_db();
    db::init_tables(&conn);

    // // For testing use only
    // // See https://github.com/Emerson-Alexander/backlist/issues/17
    // // Begin testing block
    // db::default_settings(&conn);
    // // End testing block

    // We do not display the wait_for_interaction() screen until db
    // initialization has been completed. This stops the user from getting to
    // the program's main loop too early.
    ui::wait_for_interaction();
    assume_state(AppState::MainLoop, Some(&conn))
}

/// Asks the user to select one of the top-level app states.
///
/// # Arguments
///
/// * `conn: &Connection` - main_loop will be launching AppStates that require
/// a &Connection, so it requires one too.
///
/// # Notes
///
/// main_loop is looped so that the functions of other AppStates can just end
/// and come back here. This allows us to avoid passing the &Connection to
/// functions that don't need it.
fn main_loop(conn: &Connection) {
    loop {
        ui::print_header(AppState::MainLoop);

        assume_state(
            ui::select_app_state(&[
                AppState::ToDo,
                AppState::Shop,
                AppState::AddTask,
                AppState::AddFolder,
            ]),
            Some(conn),
        );
    }
}

fn add_folder(conn: &Connection) {
    ui::print_header(AppState::AddFolder);

    let folder = ui::request_folder_input(conn);

    match folder {
        Ok(f) => db::add_folder(conn, &f).unwrap_or_else(|err| {
            eprintln!("Problem adding folder to db: {}", err);
        }),
        Err(e) => eprintln!("Problem building folder: {}", e),
    }
}

fn add_task(conn: &Connection) {
    ui::print_header(AppState::AddTask);

    let task = ui::request_task_input(conn);

    match task {
        Ok(t) => db::add_task(conn, t),
        Err(e) => eprintln!("Problem adding task: {}", e),
    }
}

/// Shows the user their current funds and allows them to enter a custom
/// transaction.
///
/// # Arguments
///
/// * `conn: &Connection` - `ui::display_funds()` requires a &Connection, so
/// it's required here too.
fn shop(conn: &Connection) {
    ui::print_header(AppState::Shop);
    ui::display_funds(finance::calc_funds(conn));
    ui::request_transaction(conn);
    ui::display_funds(finance::calc_funds(conn));
    ui::wait_for_interaction();
}

fn to_do(conn: &Connection) {
    ui::print_header(AppState::ToDo);

    // Collect a list of all active tasks
    let mut task_list = db::read_active_tasks(conn);

    // Order the list
    task_list.sort_by(|a, b| {
        calculate_weight(b)
            .partial_cmp(&calculate_weight(a))
            .unwrap()
    });

    // Shorten the list to the top 5
    if task_list.len() > 5 {
        task_list.drain(5..);
    }

    // Record that each task has been displayed
    for task in &task_list {
        db::increment_times_shown(conn, task.id, task.times_shown);
    }

    // Calculate the bounty for each task
    let tasks_w_bounties: Vec<(Task, f64)> = task_list
        .iter()
        .map(|task| (task.clone(), finance::adjusted_value(conn, &task)))
        .collect();

    // User selects a task from the remaining list
    let (selected_task, bounty) = ui::select_task(&tasks_w_bounties);

    // Record that the task has been selected
    db::increment_times_selected(conn, selected_task.id, selected_task.times_selected);

    // Display the selected task
    ui::display_task(&selected_task);
    ui::wait_for_interaction();

    // Payout the bounty
    db::add_transaction(conn, bounty);

    // Record the task as complete
    if selected_task.repeat_interval.is_some() {
        db::reset_from_date(conn, selected_task.id);
    } else {
        db::archive_task(conn, selected_task.id);
    }
}

// fn task_selected(conn: &Connection, task: &Task) {
//     ui::display_task(task);
//     finance::payout(conn, task);
//     db::increment_times_selected(conn, task.id, task.times_selected);

//     if task.repeat_interval.is_some() {
//         db::reset_from_date(conn, task.id);
//     } else {
//         db::archive_task(conn, task.id);
//     }
// }
