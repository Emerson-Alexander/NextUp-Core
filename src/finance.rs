use crate::db;
use crate::tasks::Task;
use chrono::{Duration, Utc};
use rusqlite::{Connection, Transaction};

fn calc_monthly_tasks(conn: &Connection) -> u32 {
    let task_list = db::actual_read_all_tasks(conn);

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

fn base_value(conn: &Connection) -> f64 {
    let monthly_tasks = calc_monthly_tasks(conn);

    let target_allowance = db::read_settings(conn)[0];

    let result: f64 = (target_allowance as f64) / (monthly_tasks as f64);

    let base_value = (result * 100.0).round() / 100.0;

    base_value
}

pub fn adjusted_value(conn: &Connection, task: &Task) -> f64 {
    // This whole function is TODO
    base_value(conn)
}

pub fn payout(conn: &Connection, task: &Task) {
    let bounty = adjusted_value(conn, task);

    db::add_transaction(conn, bounty as f32);
}

pub fn calc_funds(conn: &Connection) -> f32 {
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
