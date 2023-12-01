use backlist::{add_task, connect_to_db, delete_task_by_id, read_all, Task};

fn main() {
    let task_to_add = Task {
        id: 0,
        summary: String::from("Wash dishes"),
        description: String::from("Use soap"),
    };

    let conn = connect_to_db().unwrap_or_else(|err| {
        panic!("Problem establishing connection to the database: {err}");
    });

    add_task(&conn, task_to_add);

    delete_task_by_id(&conn, 2);

    let task_list = read_all(&conn);

    for task in task_list {
        println!("Here's a task {:?}", task);
    }
}
