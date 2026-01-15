use dioxus::prelude::*;
use std::collections::BTreeSet;
use std::time::Duration;
use uuid::Uuid;

use crate::components::notification::{use_notification, NotificationDisplay};
use crate::components::tag_color::tag_colors;
use crate::components::task_item::TaskItem;
use crate::components::use_details_close::use_details_close_on_outside_click;
use crate::server::{download, upload};
use crate::category::{collect_available_categories, UNCATEGORIZED_LABEL};
use crate::task::{complete_task, Task};

#[component]
pub fn MainPage() -> Element {
    let mut tasks = use_context::<Signal<Vec<Task>>>();
    let mut editing_task = use_context::<Signal<Option<Task>>>();
    let (notifications, notification_manager) = use_notification();
    let mut selected_categories = use_signal(BTreeSet::<String>::new);
    let mut selected_uncategorized = use_signal(|| false);
    use_details_close_on_outside_click();

    let on_complete = move |task_id: Uuid| {
        let mut current_tasks = tasks.write();
        let today = chrono::Local::now().date_naive();
        complete_task(&mut current_tasks, task_id, today);
    };

    let on_edit = move |task_id: Uuid| {
        if let Some(task_to_edit) = tasks.read().iter().find(|t| t.id == task_id) {
            editing_task.set(Some(task_to_edit.clone()));
        }
    };

    let on_remove = move |task_id: Uuid| {
        tasks.write().retain(|t| t.id != task_id);
    };

    let notification_manager_upload = notification_manager.clone();
    let notification_manager_download = notification_manager.clone();

    let available_set = collect_available_categories(
        tasks
            .read()
            .iter()
            .flat_map(|task| task.categories.iter()),
    );
    let available_categories = available_set.iter().cloned().collect::<Vec<_>>();
    let selected_snapshot = selected_categories.read().clone();
    let missing_selected = selected_snapshot
        .difference(&available_set)
        .cloned()
        .collect::<Vec<_>>();

    let selected_uncategorized_snapshot = *selected_uncategorized.read();
    let filtered_tasks = {
        let current = tasks.read();
        filter_tasks(&current, &selected_snapshot, selected_uncategorized_snapshot)
    };

    rsx! {
        NotificationDisplay { notifications }
        main { class: "container",
            div { class: "button-container",
                button {
                    onclick: move |_| {
                        editing_task
                            .set(
                                Some(Task {
                                    id: Uuid::new_v4(),
                                    name: "".to_string(),
                                    date: chrono::Utc::now().date_naive(),
                                    interval: 2,
                                    growth_rate: 1.3,
                                    completed_count: 0,
                                    created_at: Some(chrono::Local::now().naive_local()),
                                    categories: Vec::new(),
                                }),
                            );
                    },
                    "新增任務"
                }
                button {
                    onclick: move |_| {
                        let mut notification_manager = notification_manager_upload.clone();
                        async move {
                            let tasks_to_upload = tasks.read().clone();
                            let mut progress_handle = notification_manager.show_progress(
                                "上傳中...".to_string(),
                                0.0,
                            );

                            for i in 0..=10 {
                                progress_handle.update(i as f32 / 10.0);
                                gloo_timers::future::sleep(Duration::from_millis(100)).await;
                            }

                            match upload(tasks_to_upload).await {
                                Ok(_) => {
                                    progress_handle.complete(
                                        "上傳成功".to_string(),
                                        true,
                                    );
                                }
                                Err(e) => {
                                    progress_handle.complete(
                                        format!("上傳失敗: {e}"),
                                        false,
                                    );
                                }
                            }
                        }
                    },
                    "上傳"
                }
                button {
                    onclick: move |_| {
                        let mut notification_manager = notification_manager_download.clone();
                        async move {
                            let mut progress_handle = notification_manager.show_progress(
                                "下載中...".to_string(),
                                0.0,
                            );

                            for i in 0..=10 {
                                progress_handle.update(i as f32 / 10.0);
                                gloo_timers::future::sleep(Duration::from_millis(100)).await;
                            }

                            match download().await {
                                Ok(data) => {
                                    tasks.set(data);
                                    progress_handle.complete(
                                        "下載成功".to_string(),
                                        true,
                                    );
                                }
                                Err(e) => {
                                    progress_handle.complete(
                                        format!("下載失敗: {e}"),
                                        false,
                                    );
                                }
                            }
                        }
                    },
                    "下載"
                }
            }
            div { class: "filter-bar",
                span { class: "filter-label", "篩選分類：" }
                details { class: "filter-menu",
                    summary { class: "filter-summary",
                        span { class: "filter-summary-label",
                            {
                                if selected_snapshot.is_empty() {
                                    if selected_uncategorized_snapshot {
                                        "已選 1 項".to_string()
                                    } else {
                                        "全部".to_string()
                                    }
                                } else {
                                    let count =
                                        selected_snapshot.len() + usize::from(selected_uncategorized_snapshot);
                                    format!("已選 {} 項", count)
                                }
                            }
                        }
                    }
                    div { class: "filter-menu-list",
                        div { class: "filter-menu-actions",
                            button {
                                r#type: "button",
                                class: "filter-clear",
                                onclick: move |_| {
                                        clear_filters(selected_categories, selected_uncategorized);
                                },
                                "清除"
                            }
                            button {
                                r#type: "button",
                                class: "filter-clear",
                                onclick: move |_| {
                                    let current = selected_categories.read().clone();
                                    apply_invert_filters(
                                        selected_categories,
                                        selected_uncategorized,
                                        current,
                                        available_set.clone(),
                                        *selected_uncategorized.read(),
                                    );
                                },
                                "反選"
                            }
                        }
                        if !missing_selected.is_empty() {
                            span { class: "filter-missing-label", "已不存在：" }
                            {missing_selected.iter().map(|category| {
                                let category_for_closure = category.clone();
                                rsx!(
                                    label { class: "filter-option filter-option-missing",
                                        input {
                                            r#type: "checkbox",
                                            checked: true,
                                            onchange: move |_| {
                                                let current = selected_categories.read().clone();
                                                selected_categories.set(remove_category_selection(
                                                    &current,
                                                    &category_for_closure,
                                                ));
                                            }
                                        }
                                        span { class: "filter-swatch filter-swatch-muted" }
                                        span { class: "filter-option-text", "{category}" }
                                    }
                                )
                            })}
                        }
                        label { class: "filter-option",
                            input {
                                r#type: "checkbox",
                                checked: selected_uncategorized_snapshot,
                                onchange: move |_| {
                                    let current = *selected_uncategorized.read();
                                    selected_uncategorized.set(toggle_uncategorized(current));
                                }
                            }
                            span { class: "filter-swatch filter-swatch-muted" }
                            span { class: "filter-option-text", "{UNCATEGORIZED_LABEL}" }
                        }
                        {available_categories.iter().map(|category| {
                            let is_selected = selected_snapshot.contains(category);
                            let (bg, _fg, border) = tag_colors(category);
                            let swatch_style = format!("background: {bg}; border: 1px solid {border};");
                            let category_for_closure = category.clone();
                            rsx!(
                                label { class: "filter-option",
                                    input {
                                        r#type: "checkbox",
                                        checked: is_selected,
                                        onchange: move |_| {
                                            let current = selected_categories.read().clone();
                                            selected_categories.set(toggle_category_selection(
                                                &current,
                                                &category_for_closure,
                                            ));
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
            div { class: "task-list",
                for task in filtered_tasks {
                    TaskItem {
                        key: "{task.id}",
                        task: task.clone(),
                        on_complete,
                        on_edit,
                        on_remove,
                    }
                }
            }
        }
    }
}

fn filter_tasks(
    tasks: &[Task],
    selected_categories: &BTreeSet<String>,
    include_uncategorized: bool,
) -> Vec<Task> {
    let show_all = selected_categories.is_empty() && !include_uncategorized;
    if show_all {
        return tasks.to_vec();
    }

    tasks
        .iter()
        .filter(|task| {
            let matches_category = task
                .categories
                .iter()
                .any(|category| selected_categories.contains(category));
            let matches_uncategorized = include_uncategorized && task.categories.is_empty();
            matches_category || matches_uncategorized
        })
        .cloned()
        .collect()
}

fn invert_selection(
    selected_categories: &BTreeSet<String>,
    available_categories: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut next = selected_categories.clone();
    for category in available_categories {
        if !next.insert(category.clone()) {
            next.remove(category);
        }
    }
    next
}

fn toggle_category_selection(
    selected_categories: &BTreeSet<String>,
    category: &str,
) -> BTreeSet<String> {
    let mut next = selected_categories.clone();
    if !next.insert(category.to_string()) {
        next.remove(category);
    }
    next
}

fn remove_category_selection(
    selected_categories: &BTreeSet<String>,
    category: &str,
) -> BTreeSet<String> {
    let mut next = selected_categories.clone();
    next.remove(category);
    next
}

fn toggle_uncategorized(selected: bool) -> bool {
    !selected
}

fn clear_filters(
    mut selected_categories: Signal<BTreeSet<String>>,
    mut selected_uncategorized: Signal<bool>,
) {
    selected_categories.set(BTreeSet::new());
    selected_uncategorized.set(false);
}

fn apply_invert_filters(
    mut selected_categories: Signal<BTreeSet<String>>,
    mut selected_uncategorized: Signal<bool>,
    selected_snapshot: BTreeSet<String>,
    available_set: BTreeSet<String>,
    selected_uncategorized_snapshot: bool,
) {
    selected_categories.set(invert_selection(&selected_snapshot, &available_set));
    selected_uncategorized.set(!selected_uncategorized_snapshot);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn make_task(name: &str, categories: Vec<String>) -> Task {
        Task {
            id: Uuid::new_v4(),
            name: name.to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date"),
            interval: 1,
            growth_rate: 1.0,
            completed_count: 0,
            created_at: None,
            categories,
        }
    }

    #[test]
    fn test_filter_tasks_no_selection_returns_all() {
        let tasks = vec![
            make_task("A", vec!["work".to_string()]),
            make_task("B", Vec::new()),
        ];
        let selected = BTreeSet::new();
        let filtered = filter_tasks(&tasks, &selected, false);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_tasks_uncategorized_only() {
        let tasks = vec![
            make_task("A", vec!["work".to_string()]),
            make_task("B", Vec::new()),
        ];
        let selected = BTreeSet::new();
        let filtered = filter_tasks(&tasks, &selected, true);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "B");
    }

    #[test]
    fn test_filter_tasks_selected_category() {
        let tasks = vec![
            make_task("A", vec!["work".to_string()]),
            make_task("B", vec!["home".to_string()]),
            make_task("C", Vec::new()),
        ];
        let mut selected = BTreeSet::new();
        selected.insert("work".to_string());
        let filtered = filter_tasks(&tasks, &selected, false);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "A");
    }

    #[test]
    fn test_invert_selection_preserves_missing() {
        let mut selected = BTreeSet::new();
        selected.insert("missing".to_string());
        selected.insert("work".to_string());
        let available =
            ["work".to_string(), "home".to_string()].into_iter().collect::<BTreeSet<_>>();
        let inverted = invert_selection(&selected, &available);
        assert!(inverted.contains("missing"));
        assert!(!inverted.contains("work"));
        assert!(inverted.contains("home"));
    }

    #[test]
    fn test_toggle_category_selection_adds_and_removes() {
        let selected = BTreeSet::new();
        let selected = toggle_category_selection(&selected, "work");
        assert!(selected.contains("work"));
        let selected = toggle_category_selection(&selected, "work");
        assert!(!selected.contains("work"));
    }

    #[test]
    fn test_remove_category_selection_only_removes_target() {
        let selected =
            ["work".to_string(), "home".to_string()].into_iter().collect::<BTreeSet<_>>();
        let selected = remove_category_selection(&selected, "work");
        assert!(!selected.contains("work"));
        assert!(selected.contains("home"));
    }

    #[test]
    fn test_toggle_uncategorized_flips_value() {
        assert!(toggle_uncategorized(false));
        assert!(!toggle_uncategorized(true));
    }
}
