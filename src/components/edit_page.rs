use dioxus::prelude::*;

use crate::components::task_form::TaskForm;
use crate::task::{upsert_task, Task};

#[derive(PartialEq, Props, Clone)]
pub struct EditPageProps {
    pub task: Task,
}

#[component]
pub fn EditPage(props: EditPageProps) -> Element {
    let mut tasks = use_context::<Signal<Vec<Task>>>();
    let mut editing_task = use_context::<Signal<Option<Task>>>();

    let on_save = move |task_to_save: Task| {
        let mut current_tasks = tasks.write();
        upsert_task(&mut current_tasks, task_to_save);
        editing_task.set(None);
    };

    let on_cancel = move |_| {
        editing_task.set(None);
    };

    rsx! {
        TaskForm { task: props.task, on_save, on_cancel }
    }
}
