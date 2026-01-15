use crate::components::tag_color::tag_colors;
use crate::components::use_details_close::use_details_close_on_outside_click;
use crate::category::{collect_available_categories, normalize_category_label, normalized_set};
use crate::task::Task;
use chrono::{NaiveDate, NaiveDateTime};
use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct TaskFormProps {
    pub task: Task,
    pub on_save: EventHandler<Task>,
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn TaskForm(props: TaskFormProps) -> Element {
    let mut task_state = use_signal(|| props.task.clone());
    let mut new_category_input = use_signal(String::new);
    let formatted_date = task_state.read().date.format("%Y-%m-%d").to_string();
    let selected_categories = task_state.read().categories.clone();
    let tasks = use_context::<Signal<Vec<Task>>>();
    let mut available_tags = collect_available_categories(
        tasks
            .read()
            .iter()
            .flat_map(|task| task.categories.iter()),
    );
    available_tags.extend(normalized_set(&task_state.read().categories));
    let available_tags = available_tags.into_iter().collect::<Vec<_>>();
    let formatted_created_at = task_state
        .read()
        .created_at
        .map(|value| value.format("%Y-%m-%dT%H:%M").to_string())
        .unwrap_or_default();
    use_details_close_on_outside_click();

    rsx! {
        article {
            h3 { margin_bottom: "1rem",
                if props.task.name.is_empty() {
                    "新增任務"
                } else {
                    "編輯任務"
                }
            }
            form {
                div { class: "grid",
                    label {
                        "名稱"
                        input {
                            maxlength: "50",
                            value: "{task_state.read().name}",
                            oninput: move |evt| {
                                let mut current_task = task_state.write();
                                current_task.name = evt.value().clone();
                            },
                        }
                    }
                    label {
                        "分類"
                        div { class: "tag-input-container filter-selected-tags",
                            if selected_categories.is_empty() {
                                span { class: "filter-empty", "尚未選擇分類" }
                            } else {
                                {selected_categories.iter().map(|category| {
                                    let (bg, fg, border) = tag_colors(category);
                                    let style = format!(
                                        "background: {bg}; color: {fg}; border: 1px solid {border};"
                                    );
                                    let category_for_closure = category.clone();
                                    rsx!(
                                        span {
                                            class: "tag-pill",
                                            key: "{category}",
                                            style: "{style}",
                                            span { "{category}" }
                                            span {
                                                class: "remove-tag",
                                                onclick: move |_| {
                                                    let mut next = task_state.read().categories.clone();
                                                    next.retain(|c| c != &category_for_closure);
                                                    task_state.write().categories = next;
                                                },
                                                "×"
                                            }
                                        }
                                    )
                                })}
                            }
                        }
                        div { class: "filter-input-row",
                            input {
                                r#type: "text",
                                class: "filter-input",
                                placeholder: "新增分類...",
                                value: "{new_category_input}",
                                oninput: move |evt| new_category_input.set(evt.value()),
                                onkeydown: move |evt| {
                                    if evt.key() == Key::Enter {
                                        evt.prevent_default();
                                        add_category_from_input(task_state, new_category_input);
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "filter-add",
                                onclick: move |_| {
                                    add_category_from_input(task_state, new_category_input);
                                },
                                "新增"
                            }
                        }
                        details { class: "filter-menu",
                            summary { class: "filter-summary",
                                span { class: "filter-summary-label", "選擇分類" }
                            }
                            div { class: "filter-menu-list",
                                button {
                                    r#type: "button",
                                    class: "filter-clear",
                                    onclick: move |_| {
                                        task_state.write().categories.clear();
                                    },
                                    "清除"
                                }
                                {available_tags.iter().map(|category| {
                                    let is_selected = task_state.read().categories.contains(category);
                                    let (bg, _fg, border) = tag_colors(category);
                                    let swatch_style =
                                        format!("background: {bg}; border: 1px solid {border};");
                                    let category_for_closure = category.clone();
                                    rsx!(
                                        label { class: "filter-option",
                                            input {
                                                r#type: "checkbox",
                                                checked: is_selected,
                                                onchange: move |_| {
                                                    let mut next = task_state.read().categories.clone();
                                                    if let Some(index) =
                                                        next.iter().position(|c| c == &category_for_closure)
                                                    {
                                                        next.remove(index);
                                                    } else {
                                                        next.push(category_for_closure.clone());
                                                    }
                                                    task_state.write().categories = next;
                                                }
                                            }
                                            span { class: "filter-swatch", style: "{swatch_style}" }
                                            span { class: "filter-option-text", "{category}" }
                                        }
                                    )
                                })}
                            }
                        }
                    }
                    label {
                        "日期"
                        input {
                            r#type: "date",
                            value: "{formatted_date}",
                            oninput: move |evt| {
                                if let Ok(date) = NaiveDate::parse_from_str(&evt.value(), "%Y-%m-%d") {
                                    let mut current_task = task_state.write();
                                    current_task.date = date;
                                }
                            },
                        }
                    }
                }
                div { class: "grid",
                    label {
                        "新增時間"
                        input {
                            r#type: "datetime-local",
                            value: "{formatted_created_at}",
                            oninput: move |evt| {
                                let mut current_task = task_state.write();
                                let value = evt.value();
                                if value.trim().is_empty() {
                                    current_task.created_at = None;
                                } else if let Ok(created_at) =
                                    NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M")
                                {
                                    current_task.created_at = Some(created_at);
                                }
                            },
                        }
                    }
                }
                div { class: "grid",
                    label {
                        "間隔 (天)"
                        input {
                            r#type: "number",
                            value: "{task_state.read().interval}",
                            oninput: move |evt| {
                                if let Ok(interval) = evt.value().parse::<u32>() {
                                    let mut current_task = task_state.write();
                                    current_task.interval = interval;
                                }
                            },
                        }
                    }
                    label {
                        "成長率"
                        input {
                            r#type: "number",
                            step: "0.1",
                            value: "{task_state.read().growth_rate}",
                            oninput: move |evt| {
                                if let Ok(growth_rate) = evt.value().parse::<f64>() {
                                    let mut current_task = task_state.write();
                                    current_task.growth_rate = growth_rate;
                                }
                            },
                        }
                    }
                    label {
                        "完成次數"
                        input {
                            r#type: "number",
                            value: "{task_state.read().completed_count}",
                            oninput: move |evt| {
                                if let Ok(completed_count) = evt.value().parse::<u32>() {
                                    let mut current_task = task_state.write();
                                    current_task.completed_count = completed_count;
                                }
                            },
                        }
                    }
                }

                footer { class: "form-button-group",
                    button {
                        onclick: move |_| {
                            props.on_save.call(task_state.read().clone());
                        },
                        "儲存"
                    }
                    button {
                        class: "secondary",
                        onclick: move |_| {
                            props.on_cancel.call(());
                        },
                        "取消"
                    }
                }
            }
        }
    }
}

fn add_category_from_input(mut task_state: Signal<Task>, mut input: Signal<String>) {
    let raw = input.read().trim().to_string();
    if let Some(new_tag) = normalize_category_label(&raw) {
        let mut next = task_state.read().categories.clone();
        let normalized_existing = normalized_set(&next);
        if !normalized_existing.contains(&new_tag) {
            next.push(new_tag);
            task_state.write().categories = next;
        }
        input.set(String::new());
    }
}
