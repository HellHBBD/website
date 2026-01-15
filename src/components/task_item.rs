use crate::components::use_gesture::use_gesture;
use crate::components::tag_color::tag_colors;
use crate::category::UNCATEGORIZED_LABEL;
use crate::task::Task;
use chrono::Local;
use dioxus::prelude::*;
use log::info;
use uuid::Uuid;

#[derive(PartialEq, Props, Clone)]
pub struct TaskItemProps {
    pub task: Task,
    pub on_complete: EventHandler<Uuid>,
    pub on_edit: EventHandler<Uuid>,
    pub on_remove: EventHandler<Uuid>,
}

#[component]
pub fn TaskItem(props: TaskItemProps) -> Element {
    let task_id = props.task.id;
    let on_remove = props.on_remove;
    let on_complete = props.on_complete;
    let on_edit = props.on_edit;

    let (gesture_handlers, gesture_state) = use_gesture(
        move || {
            info!("Swiped Left on task {task_id}, removing.");
            on_remove.call(task_id);
        },
        move || {
            info!("Swiped Right on task {task_id}, completing.");
            on_complete.call(task_id);
        },
        move || {
            info!("Double-clicked on task {task_id}, editing.");
            on_edit.call(task_id);
        },
    );

    let today = Local::now().date_naive();
    let is_due_today = props.task.date <= today;
    let formatted_date = props.task.date.format("%m-%d").to_string();
    let categories = if props.task.categories.is_empty() {
        vec![UNCATEGORIZED_LABEL.to_string()]
    } else {
        props.task.categories.clone()
    };

    let dynamic_style = {
        let state = gesture_state();
        format!("transform: translateX({}px);", state.drag_x)
    };

    rsx! {
        article {
            key: "{task_id}",
            class: "task-item {is_due_today.then_some(\"task-item-due\").unwrap_or(\"\")}",
            style: "{dynamic_style}",
            ontouchstart: gesture_handlers.on_touch_start,
            ontouchmove: gesture_handlers.on_touch_move,
            ontouchend: gesture_handlers.on_touch_end,
            onmousedown: gesture_handlers.on_mouse_down,
            onmousemove: gesture_handlers.on_mouse_move,
            onmouseup: gesture_handlers.on_mouse_up,
            onmouseleave: gesture_handlers.on_mouse_leave,

            div { class: "task-item-row",
                div { class: "task-item-left",
                    strong { "{props.task.name}" }
                    div { class: "task-tag-list",
                        {categories.iter().map(|category| {
                            let is_uncategorized = category == UNCATEGORIZED_LABEL;
                            let class = if is_uncategorized {
                                "task-tag tag-uncategorized"
                            } else {
                                "task-tag"
                            };
                            let style = if is_uncategorized {
                                String::new()
                            } else {
                                let (bg, fg, border) = tag_colors(category);
                                format!(
                                    "background: {bg}; color: {fg}; border: 1px solid {border};"
                                )
                            };
                            rsx!(
                                span {
                                    class: "{class}",
                                    key: "{category}",
                                    style: "{style}",
                                    "{category}"
                                }
                            )
                        })}
                    }
                }
                div { class: "task-item-right",
                    small { class: "task-item-meta",
                        span { class: "task-item-date", "{formatted_date}" }
                        span { class: "task-item-sep", "•" }
                        span { class: "task-item-count", "{props.task.completed_count}" }
                    }
                }
            }
        }
    }
}
