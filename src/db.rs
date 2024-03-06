use core::panic;
use std::collections::HashMap;
use std::io;

use super::folders::{Folder, Style};
use super::tasks::{Priority, Task};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, params_from_iter, Connection, Error, OptionalExtension, Result, Statement};

/// Establishes connection to the SQLite db.
///
/// # Returns
///
/// `conn: Connection` will allow the rest to the program to access the db.
///
/// # Panics
///
/// May painc if it is unable to establish a connection. This will **not** occur if
/// the file does not exist. In such case, the file will be created.
pub fn connect_to_db() -> Connection {
    const DB_PATH: &str = "upNext.db";

    let conn = match Connection::open(DB_PATH) {
        Ok(file) => file,
        Err(e) => panic!("Problem establishing connection to the database: {e}"),
    };

    conn
}

/// Calls helper functions to init each table in the db
///
/// # Arguments
///
/// * `conn: Connection` - Allows helper functions to access the SQLite db.
///
/// # Panics
///
/// May panic if there are issues executing the command. I believe this would
/// only occur if there is an issue with `conn`.
pub fn init_tables(conn: &Connection) {
    init_tasks(conn);
    init_folders(conn);
    init_transactions(conn);
    init_settings(conn);
    init_statistics(conn);
}

fn is_table_empty(table_name: &str, conn: &Connection) -> bool {
    let mut stmt = conn
        .prepare(&(String::from("SELECT COUNT(*) FROM ") + table_name))
        .unwrap();
    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

    if count == 0 {
        true
    } else {
        false
    }
}

/// If necessary, create the tasks table.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
///
/// # Panics
///
/// May panic if there are issues executing the command. I believe this would
/// only occur if there is an issue with `conn`.
fn init_tasks(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER NOT NULL,
            is_archived INTEGER NOT NULL,
            summary TEXT NOT NULL,
            description TEXT,
            average_duration TEXT,
            bounty_modifier REAL NOT NULL,
            due_date TEXT,
            from_date TEXT NOT NULL,
            lead_days INTEGER,
            priority INTEGER NOT NULL,
            repeat_interval INTEGER,
            times_selected INTEGER NOT NULL,
            times_shown INTEGER NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES folders(id)
        )",
        (),
    )
    .unwrap_or_else(|err| {
        panic!("Problem accessing tasks table: {err}");
    });
}

/// If necessary, create the folders table. Then, add a top-level folder if
/// "folders" is empty.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
///
/// # Panics
///
/// - May panic if there are issues executing the command. I believe this would
/// only occur if there is an issue with `conn`.
/// - May panic if there is an issue inserting the top-level folder.
fn init_folders(conn: &Connection) {
    const DEFAULT_FOLDER_NAME: &str = "General";

    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER,
            name TEXT NOT NULL,
            style TEXT NOT NULL,
            status INTEGER,
            FOREIGN KEY (parent_id) REFERENCES folders(id)
        )",
        (),
    )
    .unwrap_or_else(|err| {
        panic!("Problem accessing folders table: {err}");
    });

    if is_table_empty("folders", conn) {
        conn.execute(
            "INSERT INTO folders (parent_id, name, style) VALUES (?, ?, ?)",
            params![None::<i64>, DEFAULT_FOLDER_NAME, "Directory"],
        )
        .unwrap_or_else(|err| {
            panic!("Problem inserting placeholder into folders table: {err}");
        });
        // TODO: Remove everything below here
        conn.execute(
            "INSERT INTO folders (parent_id, name, style) VALUES (?, ?, ?)",
            params![1, "sub-folder", "Directory"],
        )
        .unwrap_or_else(|err| {
            panic!("Problem inserting placeholder into folders table: {err}");
        });
        conn.execute(
            "INSERT INTO folders (parent_id, name, style) VALUES (?, ?, ?)",
            params![None::<i64>, "Work", "Directory"],
        )
        .unwrap_or_else(|err| {
            panic!("Problem inserting placeholder into folders table: {err}");
        });
        conn.execute(
            "INSERT INTO folders (parent_id, name, style) VALUES (?, ?, ?)",
            params![2, "sub-sub-folder", "Directory"],
        )
        .unwrap_or_else(|err| {
            panic!("Problem inserting placeholder into folders table: {err}");
        });
    }
}

/// If necessary, create the transactions table.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
///
/// # Panics
///
/// May panic if there are issues executing the command. I believe this would
/// only occur if there is an issue with `conn`.
fn init_transactions(conn: &Connection) {
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
        panic!("Problem accessing transactions table: {err}");
    });
}

/// If necessary, create the settings table. Then, add the default settings if
/// they don't already exist.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
///
/// # Panics
///
/// - May panic if there are issues executing the command. I believe this would
/// only occur if there is an issue with `conn`.
/// - May panic if there is an issue inserting the default settings.
///
/// # Note
///
/// This table is acting as a simple key-value noSQL database.
fn init_settings(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY,
            key TEXT NOT NULL,
            value TEXT NOT NULL
        )",
        (),
    )
    .unwrap_or_else(|err| {
        panic!("Problem accessing settings table: {err}");
    });

    if is_table_empty("settings", conn) {
        let default_settings = vec![
            ("maximum_monthly_allowance", 600),
            ("target_monthly_allowance", 400),
        ];

        for (key, value) in default_settings {
            conn.execute(
                "INSERT INTO settings (id, key, value) VALUES (?, ?, ?)",
                params![None::<i64>, key, value],
            )
            .unwrap_or_else(|err| {
                panic!("Problem inserting default data into settings table: {err}");
            });
        }
    }
}

/// If necessary, create the statistics table. Then, add the default statistics
/// if they don't already exist.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
///
/// # Panics
///
/// - May panic if there are issues executing the command. I believe this would
/// only occur if there is an issue with `conn`.
/// - May panic if there is an issue inserting the default statistics.
///
/// # Note
///
/// This table is acting as a simple key-value noSQL database.
fn init_statistics(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS statistics (
            id INTEGER PRIMARY KEY,
            key TEXT NOT NULL,
            value TEXT
        )",
        (),
    )
    .unwrap_or_else(|err| {
        panic!("Problem accessing folders table: {err}");
    });

    if is_table_empty("statistics", conn) {
        let default_statistics = vec![
            ("funds_unlocked", Some(0)),
            ("funds_loaded", Some(400)),
            ("average_completion_seconds", Some(600)),
            ("baseline_bounty", None),
            ("total_tasks_completed", Some(0)),
        ];

        for (key, value) in default_statistics {
            conn.execute(
                "INSERT INTO statistics (id, key, value) VALUES (?, ?, ?)",
                params![None::<i64>, key, value],
            )
            .unwrap_or_else(|err| {
                panic!("Problem inserting default data into statistics table: {err}");
            });
        }
    }
}

/// Add a Task to the tasks table.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
/// * `task: Task` - The task to add.
///
/// # Panics
///
/// May panic if there are issues executing the sql.
pub fn add_task(conn: &Connection, task: Task) {
    // rusqlite can't convert chrono::Duration
    let average_duration: Option<i64> = match task.average_duration {
        Some(d) => Some(d.num_seconds()),
        None => None,
    };

    // rusqlite can't convert custom enums
    let priority: u8 = match task.priority {
        Priority::P0 => 0,
        Priority::P1 => 1,
        Priority::P2 => 2,
        Priority::P3 => 3,
    };

    conn.execute(
        "INSERT INTO tasks (
            parent_id,
            is_archived,
            summary,
            description,
            average_duration,
            bounty_modifier,
            due_date,
            from_date,
            lead_days,
            priority,
            repeat_interval,
            times_selected,
            times_shown
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            task.parent_id,
            task.is_archived,
            task.summary,
            task.description,
            average_duration,
            task.bounty_modifier,
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

/// Add a Folder to the folders table.
///
/// # Arguments
///
/// * `conn: &Connection` - Allows us to access the SQLite db.
/// * `folder: &Folder` - The folder to add.
///
/// # Returns
///
/// Result indicating success or containing an error.
pub fn add_folder(conn: &Connection, folder: &Folder) -> Result<()> {
    conn.execute(
        "INSERT INTO folders (
            parent_id,
            name,
            style,
            status
        ) VALUES (?, ?, ?, ?)",
        params![
            folder.parent_id,
            folder.name,
            folder.style.to_string(),
            folder.status
        ],
    )?;

    Ok(())
}

// Function to recursively fetch and print the nested rows
pub fn read_all_folders(
    conn: &Connection,
    parent_id: Option<u32>,
    prefix: String,
) -> Result<HashMap<u32, String>, Error> {
    let mut stmt = conn.prepare("SELECT id, parent_id, name FROM folders WHERE parent_id IS ?")?;
    let item_iter = stmt.query_map(params![parent_id], |row| {
        Ok(Folder {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            name: row.get(2)?,
            style: Style::Directory, // TODO: set
            status: None,            // TODO: set
        })
    })?;

    let mut folders_hm: HashMap<u32, String> = HashMap::new();

    for item in item_iter {
        let item = item?;
        let new_prefix = if prefix.is_empty() {
            item.name.clone()
        } else {
            format!("{}::{}", prefix, item.name)
        };

        // println!("({}, {})", item.id, new_prefix);
        folders_hm.insert(item.id, new_prefix.clone());

        // Recursively fetch children
        // read_all_folders(conn, Some(item.id), new_prefix)?;
        folders_hm.extend(read_all_folders(conn, Some(item.id), new_prefix)?);
    }

    Ok(folders_hm)
}

pub fn add_transaction(conn: &Connection, price: f64) {
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

/// Retrieves the IDs of all descendants of the given parent_id, including those at deeper nesting levels.
///
/// # Arguments
/// * `conn: &Connection` - A reference to the SQLite connection.
/// * `parent_id: u32` - The ID of the parent for which descendant IDs are sought.
///
/// # Returns
/// * A `Result` containing a vector of descendant IDs or an error if the query fails.
pub fn get_descendant_ids(conn: &Connection, parent_id: u32) -> Result<Vec<u32>> {
    // Define a recursive Common Table Expression (CTE) to find all descendants
    let sql = "
    WITH RECURSIVE descendants(id) AS (
        SELECT id FROM folders WHERE parent_id = ?
        UNION ALL
        SELECT folders.id FROM folders, descendants WHERE folders.parent_id = descendants.id
    )
    SELECT id FROM descendants;
    ";

    // Prepare and execute the query, collecting the results
    let mut stmt = conn.prepare(sql)?;
    let descendant_ids = stmt
        .query_map(params![parent_id], |row| row.get(0))?
        .collect::<Result<Vec<u32>>>()?;

    Ok(descendant_ids)
}

// fn main() -> Result<()> {
//     // Example connection to a SQLite database
//     let conn = Connection::open("my_database.db")?;

//     // Assuming you want to find all descendants of the parent with ID 2
//     let parent_id = 2;

//     match get_descendant_ids(&conn, parent_id) {
//         Ok(descendant_ids) => {
//             println!(
//                 "Descendants of parent ID {}: {:?}",
//                 parent_id, descendant_ids
//             );
//         }
//         Err(e) => {
//             println!("Failed to get descendant IDs: {}", e);
//         }
//     }

//     Ok(())
// }

/// Reads all active tasks from the db into memory.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
///
/// # Returns
///
/// A `Vec<Task>` of all tasks that are not archived and haven't been completed
/// within their repeat_interval.
pub fn read_active_tasks(conn: &Connection) -> Vec<Task> {
    // Prepare sqlite statement
    let stmt = conn
        .prepare(
            "SELECT
            id, 
            parent_id,
            is_archived,
            summary, 
            description,
            average_duration,
            bounty_modifier, 
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

    return tasks_from_stmt(stmt, false);
}

/// Reads all tasks from the db into memory.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
///
/// # Returns
///
/// A `Vec<Task>` of all tasks.
pub fn read_all_tasks(conn: &Connection) -> Vec<Task> {
    // Prepare sqlite statement
    let stmt = conn
        .prepare(
            "SELECT
            id, 
            parent_id,
            is_archived,
            summary, 
            description, 
            average_duration,
            bounty_modifier,
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

    return tasks_from_stmt(stmt, true);
}

// /// Reads all archived tasks from the db into memory.
// ///
// /// # Arguments
// ///
// /// * `conn: Connection` - Allows us to access the SQLite db.
// ///
// /// # Returns
// ///
// /// A `Vec<Task>` of all tasks that are archived.
// pub fn read_archived_tasks(conn: &Connection) -> Vec<Task> {
//     // Prepare sqlite statement
//     let stmt = conn
//         .prepare(
//             "SELECT
//             id,
//             parent_id,
//             is_archived,
//             summary,
//             description,
//             average_duration,
//             bounty_modifier,
//             due_date,
//             from_date,
//             lead_days,
//             priority,
//             repeat_interval,
//             times_selected,
//             times_shown
//         FROM tasks WHERE is_archived = 1",
//         )
//         .unwrap_or_else(|err| {
//             panic!("Problem preparing SELECT statement: {err}");
//         });

//     return tasks_from_stmt(stmt, true);
// }

// pub fn read_active_tasks(conn: &Connection) -> Vec<Task> {
//     // Prepare sqlite statement
//     let mut stmt = conn
//         .prepare(
//             "SELECT
//             id,
//             is_archived,
//             summary,
//             description,
//             due_date,
//             from_date,
//             lead_days,
//             priority,
//             repeat_interval,
//             times_selected,
//             times_shown
//         FROM tasks WHERE is_archived = 0",
//         )
//         .unwrap_or_else(|err| {
//             panic!("Problem preparing SELECT statement: {err}");
//         });

//     /*
//     Just like in add_tasks(), rusqlite is pretty good at converting types. I
//     just need to do some pre-processing for tasks::Priority. Again, it would be
//     better to just write a macro to handle this.
//     */
//     let rows = stmt
//         .query_map([], |row| {
//             let priority: Priority = {
//                 if row.get(7) == Ok(0) {
//                     Priority::P0
//                 } else if row.get(7) == Ok(1) {
//                     Priority::P1
//                 } else if row.get(7) == Ok(2) {
//                     Priority::P2
//                 } else if row.get(7) == Ok(3) {
//                     Priority::P3
//                 } else {
//                     Priority::P1
//                 }
//             };

//             Ok(Task {
//                 id: row.get(0)?,
//                 is_archived: row.get(1)?,
//                 summary: row.get(2)?,
//                 description: row.get(3)?,
//                 due_date: row.get(4)?,
//                 from_date: row.get(5)?,
//                 lead_days: row.get(6)?,
//                 priority: priority,
//                 repeat_interval: row.get(8)?,
//                 times_selected: row.get(9)?,
//                 times_shown: row.get(10)?,
//             })
//         })
//         .unwrap_or_else(|err| {
//             panic!("Problem running SELECT statement or processing results: {err}");
//         });

//     // Converting it from a rusqlite MappedRows<Task> to a Vec<Task>.
//     let mut query_result_as_vec: Vec<Task> = Vec::new();
//     for row in rows {
//         let task = row.unwrap_or_else(|err| {
//             panic!("Problem unwrapping row after SELECT query: {err}");
//         });

//         // Only push tasks that should be added to the backlog
//         if task.repeat_interval.is_none()
//             || task.from_date + Duration::days(task.repeat_interval.unwrap_or(0) as i64)
//                 < <Utc>::now()
//         {
//             query_result_as_vec.push(task)
//         }
//     }

//     query_result_as_vec
// }

/// Fetches Tasks from the database where `parent_id` matches any u32 in the given vector.
///
/// # Arguments
///
/// * `conn: &Connection` - A reference to the SQLite connection.
/// * `parent_ids: Vec<u32>` - A vector of `u32` representing parent IDs to query for.
///
/// # Returns
///
/// A result containing a vector of tuples, each representing a row from the database,
/// or an error if the query fails.
///
/// # Examples
///
/// ```text
/// let mut conn = Connection::open("my_database.db").unwrap();
/// let parent_ids = vec![1, 2, 3];
/// let rows = fetch_by_parent_ids(&mut conn, &parent_ids).unwrap();
/// for row in rows {
///     println!("{:?}", row);
/// }
/// ```
pub fn fetch_tasks_by_parent_ids(conn: &Connection, parent_ids: Vec<u32>) -> Result<Vec<Task>> {
    // Prepare the SQL query using parameterized placeholders.
    // The number of placeholders must match the number of parent_ids.
    // Produces an output like `SELECT * FROM my_table WHERE parent_id IN (?, ?, ?).`
    let query = format!(
        "SELECT * FROM tasks WHERE parent_id IN ({})",
        parent_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Prepare the statement.
    let mut stmt = conn.prepare(&query)?;

    // Convert `parent_ids` to a dynamic type that `rusqlite` can use for the query.
    // We use `params_from_iter` to convert the vector into a suitable parameter list.
    let params = params_from_iter(parent_ids.iter());

    // Execute the query and map the results to a Vec of tuples (or whatever your row structure is).
    let rows = stmt
        .query_map(params, |row| {
            let (average_duration, priority) = convert_fields_from_sql(row.get(5)?, row.get(10)?);

            Ok(Task {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                is_archived: row.get(2)?,
                summary: row.get(3)?,
                description: row.get(4)?,
                average_duration: average_duration,
                bounty_modifier: row.get(6)?,
                due_date: row.get(7)?,
                from_date: row.get(8)?,
                lead_days: row.get(9)?,
                priority: priority,
                repeat_interval: row.get(11)?,
                times_selected: row.get(12)?,
                times_shown: row.get(13)?,
            })
        })?
        .collect();

    rows
}

fn convert_fields_from_sql(
    average_duration_row: Option<u32>,
    priority_row: u32,
) -> (Option<Duration>, Priority) {
    let average_duration = match average_duration_row {
        Some(d) => Some(Duration::seconds(d as i64)),
        None => None,
    };

    let priority: Priority = {
        if priority_row == 0 {
            Priority::P0
        } else if priority_row == 1 {
            Priority::P1
        } else if priority_row == 2 {
            Priority::P2
        } else if priority_row == 3 {
            Priority::P3
        } else {
            Priority::P1
        }
    };

    (average_duration, priority)
}

/// Helper function to query any statement that should result in a list of
/// tasks.
///
/// # Arguments
///
/// * `mut stmt: Statement<'_>` - The statement to be queried.
/// * `include_inactive: bool` - Set true to include tasks that have been
/// completed recently and have not passed their repeat_interval since.
///
/// # Returns
///
/// A `Vec<Task>` of all tasks based on the stmt and include_inactive values
/// provided.
///
/// # Notes
///
/// rusqlite uses some strange types that I'm struggling to fully wrap my head
/// around. There's a good chance that this function could be rewritten more
/// effectively.
fn tasks_from_stmt(mut stmt: Statement<'_>, include_inactive: bool) -> Vec<Task> {
    let rows = stmt
        .query_map([], |row| {
            // let average_duration = match row.get(5) {
            //     Ok(Some(d)) => Some(Duration::seconds(d)),
            //     Ok(None) => None,
            //     Err(_) => None,
            // };

            // let priority: Priority = {
            //     if row.get(10) == Ok(0) {
            //         Priority::P0
            //     } else if row.get(10) == Ok(1) {
            //         Priority::P1
            //     } else if row.get(10) == Ok(2) {
            //         Priority::P2
            //     } else if row.get(10) == Ok(3) {
            //         Priority::P3
            //     } else {
            //         Priority::P1
            //     }
            // };

            let (average_duration, priority) = convert_fields_from_sql(row.get(5)?, row.get(10)?);

            Ok(Task {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                is_archived: row.get(2)?,
                summary: row.get(3)?,
                description: row.get(4)?,
                average_duration: average_duration,
                bounty_modifier: row.get(6)?,
                due_date: row.get(7)?,
                from_date: row.get(8)?,
                lead_days: row.get(9)?,
                priority: priority,
                repeat_interval: row.get(11)?,
                times_selected: row.get(12)?,
                times_shown: row.get(13)?,
            })
        })
        .unwrap_or_else(|err| {
            panic!("Problem running SELECT statement or processing results: {err}");
        });

    // Converting it from a rusqlite MappedRows<Task> to a Vec<Task>.
    let mut query_result_as_vec: Vec<Task> = Vec::new();
    for row in rows {
        let task = row.unwrap_or_else(|err| {
            panic!("Problem unwrapping row after SELECT query: {err}");
        });

        // Only push tasks that should be added
        if task.repeat_interval.is_none()
            || task.from_date + Duration::days(task.repeat_interval.unwrap_or(0) as i64)
                < <Utc>::now()
            || include_inactive
        {
            query_result_as_vec.push(task)
        }
    }

    query_result_as_vec
}

/// TODO: Doc comment. I got it working, I need to take a break.
pub fn read_target_allowance(conn: &Connection) -> Result<u32, Error> {
    let sql = "SELECT value FROM settings WHERE key = ?1";

    let value: Option<String> = conn
        .query_row(sql, ["target_monthly_allowance"], |row| row.get(0))
        .optional()?;

    match value {
        Some(v) => v
            .parse::<u32>()
            .map_err(|_| Error::InvalidColumnName(String::from("Failed to parse TEXT to u32"))),
        None => Err(Error::QueryReturnedNoRows),
    }
}

pub fn read_transactions(conn: &Connection) -> Vec<(DateTime<Utc>, Option<f64>, Option<f64>)> {
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
    let mut query_result_as_vec: Vec<(DateTime<Utc>, Option<f64>, Option<f64>)> = Vec::new();
    for row in rows {
        let transaction = row.unwrap_or_else(|err| {
            panic!("Problem unwrapping row after SELECT query: {err}");
        });

        query_result_as_vec.push(transaction)
    }

    query_result_as_vec
}

// pub fn delete_task_by_id(conn: &Connection, id: u32) {
//     conn.execute("DELETE FROM tasks WHERE id=?1", [&id])
//         .unwrap_or_else(|err| {
//             panic!("Problem deleting task {id} from table: {err}");
//         });
// }

/// Incriments a task's times_shown by 1 in the db.
///
/// # Arguments
///
/// * `conn: Connection` - Allows us to access the SQLite db.
/// * `id: u32` - The id for the affected task.
/// * `times_shown` - The current value to be incremented (before adding 1)
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
    use std::collections::HashMap;

    use super::*;
    use chrono::TimeZone;
    use rusqlite::Result;

    #[test]
    fn test_init_tables() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn);

        // Verify table creation
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let res_tables: Result<Vec<String>> =
            stmt.query_map([], |row| row.get(0)).unwrap().collect();

        let tables = res_tables.unwrap();

        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"folders".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(!tables.contains(&"does_not_exist".to_string()));

        // Verify the initial folder insertion
        let mut stmt = conn
            .prepare("SELECT name FROM folders WHERE id = '1'")
            .unwrap();
        let folder_exists: bool = stmt.query_row((), |_| Ok(true)).is_ok();
        assert!(folder_exists, "The initial folder should be inserted.");
    }

    // Setup function to create an in-memory database and initialize the tasks table
    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_tasks(&conn);
        init_folders(&conn);
        conn
    }

    fn as_all_task_types(key_stub: String, input_task: Task) -> HashMap<String, Task> {
        let one_off = Task {
            due_date: None,
            lead_days: None,
            repeat_interval: None,
            ..input_task.clone()
        };
        let due = Task {
            due_date: Some(Utc.timestamp_opt(1234567890, 0).unwrap()),
            lead_days: Some(3),
            repeat_interval: None,
            ..input_task.clone()
        };
        let repeat = Task {
            due_date: None,
            lead_days: None,
            repeat_interval: Some(7),
            ..input_task.clone()
        };

        let mut tasks = HashMap::new();
        tasks.insert(key_stub.clone() + "_one_off", one_off);
        tasks.insert(key_stub.clone() + "_due", due);
        tasks.insert(key_stub.clone() + "_repeat", repeat);

        tasks
    }

    // Generate training tasks
    fn generate_training_tasks() -> HashMap<String, Task> {
        let mut tasks = HashMap::new();

        let all_fields_full = Task {
            id: 0, // This will be ignored by add_task()
            parent_id: 1,
            is_archived: false,
            summary: "Test task".into(),
            description: Some("Test description".into()),
            average_duration: Some(Duration::seconds(3600)),
            bounty_modifier: 1.0,
            due_date: Some(Utc.timestamp_opt(1234567890, 0).unwrap()),
            from_date: Utc.timestamp_opt(1234567890, 0).unwrap(),
            lead_days: Some(3),
            priority: Priority::P1,
            repeat_interval: Some(7),
            times_selected: 5,
            times_shown: 10,
        };
        tasks.insert(String::from("all fields full"), all_fields_full.clone());

        tasks.insert(
            String::from("all_optional_fields_empty"),
            Task {
                description: None,
                average_duration: None,
                due_date: None,
                lead_days: None,
                repeat_interval: None,
                ..all_fields_full.clone()
            },
        );

        tasks.extend(as_all_task_types(
            String::from("basic"),
            Task {
                ..all_fields_full.clone()
            },
        ));

        tasks.extend(as_all_task_types(
            String::from("is_archived_true"),
            Task {
                is_archived: true,
                ..all_fields_full.clone()
            },
        ));

        tasks.extend(as_all_task_types(
            String::from("priority_0"),
            Task {
                priority: Priority::P0,
                ..all_fields_full.clone()
            },
        ));
        tasks.extend(as_all_task_types(
            String::from("priority_2"),
            Task {
                priority: Priority::P2,
                ..all_fields_full.clone()
            },
        ));
        tasks.extend(as_all_task_types(
            String::from("priority_3"),
            Task {
                priority: Priority::P3,
                ..all_fields_full.clone()
            },
        ));

        tasks.extend(as_all_task_types(
            String::from("bounty_mod_0"),
            Task {
                bounty_modifier: 0.0,
                ..all_fields_full.clone()
            },
        ));
        tasks.extend(as_all_task_types(
            String::from("bounty_mod_negative"),
            Task {
                bounty_modifier: -1.0,
                ..all_fields_full.clone()
            },
        ));
        tasks.extend(as_all_task_types(
            String::from("bounty_mod_less_than_1"),
            Task {
                bounty_modifier: 0.3,
                ..all_fields_full.clone()
            },
        ));
        tasks.extend(as_all_task_types(
            String::from("bounty_mod_more_than_1"),
            Task {
                bounty_modifier: 1.7,
                ..all_fields_full.clone()
            },
        ));
        tasks.extend(as_all_task_types(
            String::from("bounty_mod_more_than_2"),
            Task {
                bounty_modifier: 5.6,
                ..all_fields_full.clone()
            },
        ));

        tasks
    }

    #[test]
    fn test_add_task() {
        let conn = setup_db();

        let tasks_input = generate_training_tasks();

        for (_, task) in tasks_input.clone() {
            add_task(&conn, task);
        }

        // Verify that the task was inserted correctly
        let mut stmt = conn.prepare("SELECT * FROM tasks").unwrap();
        let tasks_output = stmt.query_map((), |_| Ok(())).unwrap();

        assert_eq!(tasks_output.count(), tasks_input.len());
    }
}
