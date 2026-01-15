use dioxus::prelude::*;

use crate::components::edit_page::EditPage;
use crate::components::main_page::MainPage;
use crate::server::download;
use crate::task::Task;

#[component]
pub fn SchedulePage() -> Element {
    let editing_task = use_context::<Signal<Option<Task>>>();
    let mut tasks = use_context::<Signal<Vec<Task>>>();
    let task_snapshot = editing_task.read().clone();
    use_effect(move || {
        spawn(async move {
            if let Ok(downloaded_tasks) = download().await {
                tasks.set(downloaded_tasks);
            }
        });
    });
    rsx! {
        document::Title { "schedule" }
        div { class: "home schedule-page",
            div { class: "home-bg" }
            section { class: "schedule-content",
                {
                    match task_snapshot {
                        Some(task) => rsx! {
                            EditPage { task }
                        },
                        None => rsx! {
                            MainPage {}
                        },
                    }
                }
            }
        }
    }
}
