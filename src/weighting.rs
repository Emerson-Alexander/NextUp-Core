use super::tasks::{Priority, Task};
use chrono::{DateTime, Duration, Utc};

pub fn calculate_weight(task: &Task) -> f32 {
    // I'm sure there is a more elegant way to structure this logic in Rust
    match task.due_date {
        Some(_) => weight_due_task(task),
        None => match task.repeat_interval {
            Some(_) => weight_repeat_task(task),
            None => weight_oneoff_task(task),
        },
    }
}

fn weight_due_task(task: &Task) -> f32 {
    let mut weight: f32;

    if DateTime::<Utc>::timestamp(&Utc::now())
        <= DateTime::<Utc>::timestamp(&task.due_date.unwrap())
            - (task.lead_days.unwrap() as i64 * 86400)
    {
        // y = now / ( due_date - lead_days[as seconds] )
        weight = DateTime::<Utc>::timestamp(&Utc::now()) as f32
            / (DateTime::<Utc>::timestamp(&task.due_date.unwrap()) as f32
                - (task.lead_days.unwrap() as f32 * 86400.0));
    } else {
        // y = 1 + 100(now-due_date+lead_days[as seconds])/lead_days[as seconds]
        // this will panic if you have a due date and no lead days... very non-rust
        weight = (100.0
            * ((DateTime::<Utc>::timestamp(&Utc::now()) as f32)
                - (DateTime::<Utc>::timestamp(&task.due_date.unwrap()) as f32)
                + (task.lead_days.unwrap() as f32 * 86400.0))
            / (task.lead_days.unwrap() as f32 * 86400.0))
            + 1.0;
    }

    weight = weight * adjust_for_priority(task);

    weight
}

fn weight_repeat_task(task: &Task) -> f32 {
    // Returning a weight of 0.0 if the task isn't old enough to be selected
    if task.from_date + Duration::days(i64::from(task.repeat_interval.unwrap())) >= Utc::now() {
        return 0.0;
    }

    let mut weight: f32 = 1.0;
    weight = weight * adjust_for_priority(task);

    // y=0.667x+0.333 where x is the number of repeat_intervals lapsed
    weight = weight
        * (0.667
            * (DateTime::<Utc>::timestamp(&Utc::now()) as f32
                / (task.from_date + Duration::days(i64::from(task.repeat_interval.unwrap())))
                    .timestamp() as f32)
            + 0.333);

    weight
}

fn weight_oneoff_task(task: &Task) -> f32 {
    let mut weight: f32 = 1.0;
    weight = weight * adjust_for_priority(task);

    // y=0.667x+1 where x is the number of 20 day periods lapsed
    weight = weight
        * (0.667
            * (DateTime::<Utc>::timestamp(&Utc::now()) as f32
                / (task.from_date + Duration::days(20)).timestamp() as f32)
            + 1.0);

    weight
}

fn adjust_for_priority(task: &Task) -> f32 {
    match task.priority {
        Priority::P0 => 2.0,
        Priority::P1 => 3.0,
        Priority::P2 => 5.0,
        Priority::P3 => 8.0,
    }
}
