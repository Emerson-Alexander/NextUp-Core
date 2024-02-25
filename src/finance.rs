use crate::db;
use crate::tasks::Task;
use chrono::{Duration, Utc};
use rusqlite::{Connection, Transaction};

/// Determines the average number of tasks the user can expect to complete in a
/// month.
///
/// # Arguments
///
/// * `conn: &Connection` - Allows connection to the db.
///
/// # Returns
///
/// A `u32` of the expected number of tasks.
///
/// # Notes
///
/// This includes all recurring tasks weighted by their repeat_interval in
/// addition to the number of one-time tasks and tasks with due dates created
/// in the last 30 days.
fn calc_monthly_tasks(conn: &Connection) -> u32 {
    let task_list = db::read_all_tasks(conn);

    let mut avg_monthly_tasks = 0;

    for task in task_list {
        match task.repeat_interval {
            Some(interval) => avg_monthly_tasks += 30 / interval,
            None => {
                if task.from_date + Duration::days(3) > <Utc>::now() {
                    avg_monthly_tasks += 1;
                }
            }
        }
    }

    avg_monthly_tasks
}

/// Calculate the payout for the average task, before any weighting.
///
/// # Arguments
///
/// * `conn: &Connection` - Allows connection to the db.
///
/// # Returns
///
/// An `f64` of the expected payout.
fn base_value(conn: &Connection) -> f64 {
    // Determine how many tasks will be completed each month and how much the
    // user hopes to add to their budget.
    let monthly_tasks = calc_monthly_tasks(conn);
    let target_allowance = db::read_settings(conn)[0];

    // Divide the factors
    let result: f64 = (target_allowance as f64) / (monthly_tasks as f64);

    // Round the result to 2 decimal places
    let base_value = (result * 100.0).round() / 100.0;

    base_value
}

/// Will eventually calculate an individual payout for each task based on the
/// number of times shown vs times selected. For now it just passes through the
/// base_value of all tasks.
pub fn adjusted_value(conn: &Connection, task: &Task) -> f64 {
    // This whole function is TODO
    base_value(conn)
}

pub fn payout(conn: &Connection, task: &Task) {
    let bounty = adjusted_value(conn, task);

    db::add_transaction(conn, bounty as f64);
}

pub fn calc_funds(conn: &Connection) -> f64 {
    let transactions = db::read_transactions(conn);

    let mut total_funds = 0.0;

    for transaction in transactions {
        match transaction.1 {
            Some(v) => total_funds += v,
            None => total_funds -= transaction.2.unwrap(),
        }
    }

    total_funds
}
