use rusqlite::{params, Connection, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: i32,
    pub summary: String,
    pub description: String,
}

pub fn connect_to_db() -> Result<Connection> {
    let conn = Connection::open("tasks.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            summary TEXT NOT NULL,
            description TEXT NOT NULL
        )",
        (),
    )?;

    Ok(conn)
}

pub fn read_all(conn: &Connection) -> Vec<Task> {
    // Running SQL query
    let mut stmt = conn
        .prepare("SELECT id, summary, description FROM tasks")
        .unwrap_or_else(|err| {
            panic!("Problem reading from table: {err}");
        });

    // Mapping the query result into something more usable
    let task_iter = stmt
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                summary: row.get(1)?,
                description: row.get(2)?,
            })
        })
        .unwrap_or_else(|err| {
            panic!("Problem unwraping read query to iter: {err}");
        });

    // Converting it from a rusqlite collection to a Vec<Task>
    // There might be a better way to do this if I was better at Rust
    let mut query_result_as_vec: Vec<Task> = Vec::new();
    for task in task_iter {
        query_result_as_vec.push(task.unwrap())
    }

    query_result_as_vec
}

pub fn add_task(conn: &Connection, task: Task) {
    conn.execute(
        "INSERT INTO tasks (summary, description) VALUES (?1, ?2)",
        (&task.summary, &task.description),
    )
    .unwrap_or_else(|err| {
        panic!("Problem adding task to table: {err}");
    });
}

pub fn delete_task_by_id(conn: &Connection, id: i32) {
    conn.execute("DELETE FROM tasks WHERE id=?1", params![&id])
        .unwrap_or_else(|err| {
            panic!("Problem deleting task {id} from table: {err}");
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_db() -> Connection {
        // Connecting to in-memory sqlite database
        let conn = Connection::open_in_memory().unwrap_or_else(|err| {
            panic!("Problem establishing connection to the database: {err}");
        });

        // Creating a table
        conn.execute(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                summary TEXT NOT NULL,
                description TEXT NOT NULL
            )",
            (),
        )
        .unwrap_or_else(|err| {
            panic!("Problem creating table: {err}");
        });

        // Hardcoding testing data to add to our test database
        let mut test_tasks: Vec<Task> = Vec::new();
        test_tasks.push(Task {
            id: 1,
            summary: String::from("Fead dog"),
            description: String::from("Give dog food"),
        });
        test_tasks.push(Task {
            id: 2,
            summary: String::from("Fead cat"),
            description: String::from("Give cat food"),
        });
        test_tasks.push(Task {
            id: 3,
            summary: String::from("Take out trash"),
            description: String::from("Don't forget recycling!"),
        });

        // Inserting testing data into the table
        for task in test_tasks {
            conn.execute(
                "INSERT INTO tasks (summary, description) VALUES (?1, ?2)",
                (&task.summary, &task.description),
            )
            .unwrap_or_else(|err| {
                panic!("Problem inserting task into table: {err}");
            });
        }

        // Returning our connection to the test database
        conn
    }

    #[test]
    fn test_read_all() {
        let task_vec = read_all(&init_test_db());

        // Recreating testing data from init_test_db()
        // TODO: Don't repeat this code
        let mut test_tasks: Vec<Task> = Vec::new();
        test_tasks.push(Task {
            id: 1,
            summary: String::from("Fead dog"),
            description: String::from("Give dog food"),
        });
        test_tasks.push(Task {
            id: 2,
            summary: String::from("Fead cat"),
            description: String::from("Give cat food"),
        });
        test_tasks.push(Task {
            id: 3,
            summary: String::from("Take out trash"),
            description: String::from("Don't forget recycling!"),
        });

        assert_eq!(test_tasks, task_vec);
    }

    #[test]
    fn test_add_task() {
        let task_to_add = Task {
            id: 4,
            summary: String::from("Clean fridge"),
            description: String::from("Remove old food"),
        };

        // Recreating testing data from init_test_db()
        // TODO: Don't repeat this code
        let mut test_tasks: Vec<Task> = Vec::new();
        test_tasks.push(Task {
            id: 1,
            summary: String::from("Fead dog"),
            description: String::from("Give dog food"),
        });
        test_tasks.push(Task {
            id: 2,
            summary: String::from("Fead cat"),
            description: String::from("Give cat food"),
        });
        test_tasks.push(Task {
            id: 3,
            summary: String::from("Take out trash"),
            description: String::from("Don't forget recycling!"),
        });

        // Also adding the new task
        test_tasks.push(task_to_add.clone());

        // Working with the database
        let conn = init_test_db();
        add_task(&conn, task_to_add);
        let task_vec = read_all(&conn);

        assert_eq!(test_tasks, task_vec);
    }

    #[test]
    fn test_delete_task_by_id() {
        // Recreating testing data from init_test_db()
        // TODO: Don't repeat this code
        let mut test_tasks: Vec<Task> = Vec::new();
        test_tasks.push(Task {
            id: 1,
            summary: String::from("Fead dog"),
            description: String::from("Give dog food"),
        });
        test_tasks.push(Task {
            id: 3,
            summary: String::from("Take out trash"),
            description: String::from("Don't forget recycling!"),
        });

        // Working with the database
        let conn = init_test_db();
        delete_task_by_id(&conn, 2);
        let task_vec = read_all(&conn);

        assert_eq!(test_tasks, task_vec);
    }
}
