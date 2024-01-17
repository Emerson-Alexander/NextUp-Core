use super::tasks::{Priority, Task};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};

pub fn connect_to_db() -> Connection {
    let conn = match Connection::open("tasks.db") {
        Ok(file) => file,
        Err(e) => panic!("Problem establishing connection to the database: {e}"),
    };

    create_table(&conn);
    create_table_transactions(&conn);
    create_table_settings(&conn);

    default_settings(&conn);

    conn
}

fn create_table(conn: &Connection) {
    /*
    I've split this off into its own function so that our unit tests can create
    the exact same table in memory.
    */
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            is_archived INTEGER NOT NULL,
            summary TEXT NOT NULL,
            description TEXT,
            due_date INTEGER,
            from_date INTEGER NOT NULL,
            lead_days INTEGER,
            priority INTEGER NOT NULL,
            repeat_interval INTEGER,
            times_selected INTEGER NOT NULL,
            times_shown INTEGER NOT NULL
        )",
        (),
    )
    .unwrap_or_else(|err| {
        panic!("Problem creating table: {err}");
    });
}

fn create_table_transactions(conn: &Connection) {
    /*
    I've split this off into its own function so that our unit tests can create
    the exact same table in memory.
    */
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY,
            date INTEGER NOT NULL,
            funds_added INTEGER,
            funds_subtracted INTEGER
        )",
        (),
    )
    .unwrap_or_else(|err| {
        panic!("Problem creating table: {err}");
    });
}

fn create_table_settings(conn: &Connection) {
    /*
    I've split this off into its own function so that our unit tests can create
    the exact same table in memory.
    */
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            target_allowance INTEGER NOT NULL,
            max_allowance INTEGER NOT NULL
        )",
        (),
    )
    .unwrap_or_else(|err| {
        panic!("Problem creating table: {err}");
    });
}

pub fn add_task(conn: &Connection, task: Task) {
    /*
    rusqlite is pretty good about using params![] to convert everything into
    the necessary types. This includes turning Option<T>s into Nulls. I begin
    by converting the Priority into a u8. It would probably be better to just
    use a macro or something to avoid needing to do the conversion, but this is
    already enough of a learning project for me as is.
    */
    let priority: u8 = match task.priority {
        Priority::P0 => 0,
        Priority::P1 => 1,
        Priority::P2 => 2,
        Priority::P3 => 3,
    };

    conn.execute(
        "INSERT INTO tasks (
            is_archived,
            summary,
            description,
            due_date,
            from_date,
            lead_days,
            priority,
            repeat_interval,
            times_selected,
            times_shown
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            task.is_archived,
            task.summary,
            task.description,
            task.due_date,
            task.from_date,
            task.lead_days,
            priority,
            task.repeat_interval,
            task.times_selected,
            task.times_shown
        ],
    )
    .unwrap_or_else(|err| {
        panic!("Problem adding task to table: {err}");
    });
}

pub fn add_transaction(conn: &Connection, price: f32) {
    if price >= 0.0 {
        conn.execute(
            "INSERT INTO transactions (
                date,
                funds_added
            ) VALUES (?, ?)",
            params![<Utc>::now(), price],
        )
        .unwrap_or_else(|err| {
            panic!("Problem adding task to table: {err}");
        });
    } else {
        conn.execute(
            "INSERT INTO transactions (
                date,
                funds_subtracted
            ) VALUES (?, ?)",
            params![<Utc>::now(), price * -1.0],
        )
        .unwrap_or_else(|err| {
            panic!("Problem adding task to table: {err}");
        });
    }
}

pub fn default_settings(conn: &Connection) {
    conn.execute(
        "INSERT INTO settings (
                target_allowance,
                max_allowance
            ) VALUES (?, ?)",
        params![400.0, 500.0],
    )
    .unwrap_or_else(|err| {
        panic!("Problem adding task to table: {err}");
    });
}

// TODO: Refactor to read_active_tasks
pub fn read_all_tasks(conn: &Connection) -> Vec<Task> {
    /*
    Technically this is only reading the non-archived tasks. I may change that
    if there's ever a use case for checking the archived tasks.
    */

    // Prepare sqlite statement
    let mut stmt = conn
        .prepare(
            "SELECT
            id, 
            is_archived,
            summary, 
            description, 
            due_date, 
            from_date, 
            lead_days, 
            priority, 
            repeat_interval, 
            times_selected, 
            times_shown
        FROM tasks WHERE is_archived = 0",
        )
        .unwrap_or_else(|err| {
            panic!("Problem preparing SELECT statement: {err}");
        });

    /*
    Just like in add_tasks(), rusqlite is pretty good at converting types. I
    just need to do some pre-processing for tasks::Priority. Again, it would be
    better to just write a macro to handle this.
    */
    let rows = stmt
        .query_map([], |row| {
            let priority: Priority = {
                if row.get(7) == Ok(0) {
                    Priority::P0
                } else if row.get(7) == Ok(1) {
                    Priority::P1
                } else if row.get(7) == Ok(2) {
                    Priority::P2
                } else if row.get(7) == Ok(3) {
                    Priority::P3
                } else {
                    Priority::P1
                }
            };

            Ok(Task {
                id: row.get(0)?,
                is_archived: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                due_date: row.get(4)?,
                from_date: row.get(5)?,
                lead_days: row.get(6)?,
                priority: priority,
                repeat_interval: row.get(8)?,
                times_selected: row.get(9)?,
                times_shown: row.get(10)?,
            })
        })
        .unwrap_or_else(|err| {
            panic!("Problem running SELECT statement or processing results: {err}");
        });

    // Converting it from a rusqlite MappedRows<Task> to a Vec<Task>.
    // This might not be necessary if I was more comfortable with rusqlite.
    let mut query_result_as_vec: Vec<Task> = Vec::new();
    for row in rows {
        let task = row.unwrap_or_else(|err| {
            panic!("Problem unwrapping row after SELECT query: {err}");
        });

        // Only push tasks that should be added to the backlog
        if task.repeat_interval.is_none()
            || task.from_date + Duration::days(task.repeat_interval.unwrap_or(0) as i64)
                < <Utc>::now()
        {
            query_result_as_vec.push(task)
        }
    }

    query_result_as_vec
}

pub fn read_settings(conn: &Connection) -> [u32; 2] {
    let mut stmt = conn
        .prepare(
            "SELECT
            target_allowance,
            max_allowance
        FROM settings",
        )
        .unwrap_or_else(|err| {
            panic!("Problem preparing SELECT statement: {err}");
        });

    let result_iter = stmt
        .query_map([], |row| Ok([row.get(0).unwrap(), row.get(1).unwrap()]))
        .unwrap();

    let mut settings: [u32; 2] = [0, 0];

    for result in result_iter {
        settings = result.unwrap();
    }

    settings
}

pub fn read_transactions(conn: &Connection) -> Vec<(DateTime<Utc>, Option<f32>, Option<f32>)> {
    let mut stmt = conn
        .prepare(
            "SELECT
            date,
            funds_added,
            funds_subtracted
        FROM transactions",
        )
        .unwrap_or_else(|err| {
            panic!("Problem preparing SELECT statement: {err}");
        });

    let rows = stmt
        // .query_map([], |row| Ok([row.get(0).unwrap(), row.get(1).unwrap()]))
        .query_map([], |row| match row.get(1).unwrap() {
            Some(price) => Ok((row.get(0).unwrap(), Some(price), None)),
            None => Ok((row.get(0).unwrap(), None, Some(row.get(2).unwrap()))),
        })
        .unwrap();

    // Converting it from a rusqlite MappedRows<Task> to a Vec<Task>.
    // This might not be necessary if I was more comfortable with rusqlite.
    let mut query_result_as_vec: Vec<(DateTime<Utc>, Option<f32>, Option<f32>)> = Vec::new();
    for row in rows {
        let transaction = row.unwrap_or_else(|err| {
            panic!("Problem unwrapping row after SELECT query: {err}");
        });

        query_result_as_vec.push(transaction)
    }

    query_result_as_vec
}

pub fn actual_read_all_tasks(conn: &Connection) -> Vec<Task> {
    /*
    Technically this is only reading the non-archived tasks. I may change that
    if there's ever a use case for checking the archived tasks.
    */

    // Prepare sqlite statement
    let mut stmt = conn
        .prepare(
            "SELECT
            id, 
            is_archived,
            summary, 
            description, 
            due_date, 
            from_date, 
            lead_days, 
            priority, 
            repeat_interval, 
            times_selected, 
            times_shown
        FROM tasks",
        )
        .unwrap_or_else(|err| {
            panic!("Problem preparing SELECT statement: {err}");
        });

    /*
    Just like in add_tasks(), rusqlite is pretty good at converting types. I
    just need to do some pre-processing for tasks::Priority. Again, it would be
    better to just write a macro to handle this.
    */
    let rows = stmt
        .query_map([], |row| {
            let priority: Priority = {
                if row.get(7) == Ok(0) {
                    Priority::P0
                } else if row.get(7) == Ok(1) {
                    Priority::P1
                } else if row.get(7) == Ok(2) {
                    Priority::P2
                } else if row.get(7) == Ok(3) {
                    Priority::P3
                } else {
                    Priority::P1
                }
            };

            Ok(Task {
                id: row.get(0)?,
                is_archived: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                due_date: row.get(4)?,
                from_date: row.get(5)?,
                lead_days: row.get(6)?,
                priority: priority,
                repeat_interval: row.get(8)?,
                times_selected: row.get(9)?,
                times_shown: row.get(10)?,
            })
        })
        .unwrap_or_else(|err| {
            panic!("Problem running SELECT statement or processing results: {err}");
        });

    // Converting it from a rusqlite MappedRows<Task> to a Vec<Task>.
    // This might not be necessary if I was more comfortable with rusqlite.
    let mut query_result_as_vec: Vec<Task> = Vec::new();
    for row in rows {
        let task = row.unwrap_or_else(|err| {
            panic!("Problem unwrapping row after SELECT query: {err}");
        });

        query_result_as_vec.push(task)
    }

    query_result_as_vec
}

pub fn read_archived_tasks(conn: &Connection) -> Vec<Task> {
    /*
    Technically this is only reading the non-archived tasks. I may change that
    if there's ever a use case for checking the archived tasks.
    */

    // Prepare sqlite statement
    let mut stmt = conn
        .prepare(
            "SELECT
            id, 
            is_archived,
            summary, 
            description, 
            due_date, 
            from_date, 
            lead_days, 
            priority, 
            repeat_interval, 
            times_selected, 
            times_shown
        FROM tasks WHERE is_archived = 1",
        )
        .unwrap_or_else(|err| {
            panic!("Problem preparing SELECT statement: {err}");
        });

    /*
    Just like in add_tasks(), rusqlite is pretty good at converting types. I
    just need to do some pre-processing for tasks::Priority. Again, it would be
    better to just write a macro to handle this.
    */
    let rows = stmt
        .query_map([], |row| {
            let priority: Priority = {
                if row.get(7) == Ok(0) {
                    Priority::P0
                } else if row.get(7) == Ok(1) {
                    Priority::P1
                } else if row.get(7) == Ok(2) {
                    Priority::P2
                } else if row.get(7) == Ok(3) {
                    Priority::P3
                } else {
                    Priority::P1
                }
            };

            Ok(Task {
                id: row.get(0)?,
                is_archived: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                due_date: row.get(4)?,
                from_date: row.get(5)?,
                lead_days: row.get(6)?,
                priority: priority,
                repeat_interval: row.get(8)?,
                times_selected: row.get(9)?,
                times_shown: row.get(10)?,
            })
        })
        .unwrap_or_else(|err| {
            panic!("Problem running SELECT statement or processing results: {err}");
        });

    // Converting it from a rusqlite MappedRows<Task> to a Vec<Task>.
    // This might not be necessary if I was more comfortable with rusqlite.
    let mut query_result_as_vec: Vec<Task> = Vec::new();
    for row in rows {
        let task = row.unwrap_or_else(|err| {
            panic!("Problem unwrapping row after SELECT query: {err}");
        });

        query_result_as_vec.push(task)
    }

    query_result_as_vec
}

// pub fn delete_task_by_id(conn: &Connection, id: u32) {
//     conn.execute("DELETE FROM tasks WHERE id=?1", [&id])
//         .unwrap_or_else(|err| {
//             panic!("Problem deleting task {id} from table: {err}");
//         });
// }

pub fn increment_times_shown(conn: &Connection, id: u32, times_shown: u32) {
    conn.execute(
        "UPDATE tasks SET times_shown=?1 WHERE id=?2",
        [times_shown + 1, id],
    )
    .unwrap_or_else(|err| {
        panic!("Problem updating task: {err}");
    });
}

pub fn increment_times_selected(conn: &Connection, id: u32, times_selected: u32) {
    conn.execute(
        "UPDATE tasks SET times_selected=?1 WHERE id=?2",
        [times_selected + 1, id],
    )
    .unwrap_or_else(|err| {
        panic!("Problem updating task: {err}");
    });
}

pub fn reset_from_date(conn: &Connection, id: u32) {
    conn.execute(
        "UPDATE tasks SET from_date=? WHERE id=?",
        params![<Utc>::now(), id],
    )
    .unwrap_or_else(|err| {
        panic!("Problem updating task: {err}");
    });
}

pub fn archive_task(conn: &Connection, id: u32) {
    println!("Archiving task by id {}", &id);

    conn.execute("UPDATE tasks SET is_archived=1 WHERE id=?", params![id])
        .unwrap_or_else(|err| {
            panic!("Problem updating task: {err}");
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn connect_to_test_db() -> Connection {
        // Connecting to in-memory sqlite database
        let conn = Connection::open_in_memory().unwrap_or_else(|err| {
            panic!("Problem establishing connection to the database: {err}");
        });

        create_table(&conn);

        conn
    }

    fn generate_basic_test_data() -> Vec<Task> {
        let task_1 = Task {
            id: 1,
            is_archived: false,
            summary: String::from("Wash the dishes"),
            description: Some(String::from("Use lots of soap")),
            due_date: Some(
                DateTime::<Utc>::from_timestamp(2431648000, 0).expect("Invalid timestamp"),
            ),
            from_date: DateTime::<Utc>::from_timestamp(1431648000, 0).expect("Invalid timestamp"),
            lead_days: Some(10),
            priority: Priority::P3,
            repeat_interval: Some(50),
            times_selected: 5,
            times_shown: 15,
        };
        let task_2 = Task {
            id: 2,
            summary: String::from("Fead the cat"),
            description: None,
            due_date: None,
            lead_days: None,
            repeat_interval: None,
            ..task_1.clone()
        };
        let task_3 = Task {
            id: 3,
            summary: String::from("Take out trash"),
            is_archived: true,
            ..task_1.clone()
        };
        let task_4 = Task {
            id: 4,
            summary: String::from("Scrub the floors"),
            is_archived: true,
            ..task_2.clone()
        };
        let task_5 = Task {
            id: 5,
            summary: String::from("`~!@#$%^&*()_+-=[]12345"),
            description: Some(String::from("`~!@#$%^&*()_+-=[]12345")),
            ..task_1.clone()
        };
        let task_6 = Task {
            id: 6,
            summary: String::from("Walk the dogs"),
            times_selected: 20,
            times_shown: 20,
            ..task_1.clone()
        };
        let task_7 = Task {
            id: 7,
            summary: String::from("Clean the sink"),
            times_selected: 0,
            times_shown: 0,
            ..task_1.clone()
        };

        let tasks = vec![task_1, task_2, task_3, task_4, task_5, task_6, task_7];

        tasks
    }

    #[test]
    fn test_add_and_read_db() {
        // Prepare the in-memory db
        let conn = connect_to_test_db();
        let source_data = generate_basic_test_data();

        // Run the add function we're testing
        for task in &source_data {
            add_task(&conn, task.clone());
        }

        // Run the read function we're testing
        let test_data = read_all_tasks(&conn);

        assert_eq!(source_data, test_data);
    }

    // #[test]
    // fn test_delete_task_by_id() {
    //     // Prepare the in-memory db
    //     let conn = connect_to_test_db();
    //     let source_data = generate_basic_test_data();
    //     for task in &source_data {
    //         add_task(&conn, task.clone());
    //     }

    //     // Remove items 2, 6, and 6 from the source data
    //     let mut deleted_source_data: Vec<Task> = Vec::new();
    //     for task in source_data {
    //         if ![2, 4, 6].contains(&task.id) {
    //             deleted_source_data.push(task)
    //         }
    //     }

    //     // Run the delete function we're testing
    //     delete_task_by_id(&conn, 2);
    //     delete_task_by_id(&conn, 4);
    //     delete_task_by_id(&conn, 6);

    //     // Read from the in-memory db
    //     let test_data = read_all_tasks(&conn);

    //     assert_eq!(deleted_source_data, test_data);
    // }
}
