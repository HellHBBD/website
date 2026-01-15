use dioxus::prelude::*;

use crate::components::tag_color::tag_colors;
use crate::category::{normalize_category_label, normalized_set, UNCATEGORIZED_LABEL};

#[derive(PartialEq, Props, Clone)]
pub struct TagInputProps {
    pub initial_tags: Vec<String>,
    pub on_change: EventHandler<Vec<String>>,
    pub available_tags: Vec<String>,
}

#[component]
pub fn TagInput(props: TagInputProps) -> Element {
    let mut selected_tags = use_signal(|| props.initial_tags.clone());
    let mut current_input = use_signal(String::new);

    if selected_tags.with(|current| current != &props.initial_tags) {
        selected_tags.set(props.initial_tags.clone());
    }

    let selected_snapshot = selected_tags.read().clone();
    let normalized_selected = normalized_set(&selected_snapshot);
    let available_suggestions = {
        props
            .available_tags
            .iter()
            .filter(|tag| !selected_snapshot.contains(tag) && *tag != UNCATEGORIZED_LABEL)
            .cloned()
            .collect::<Vec<_>>()
    };

    rsx! {
        div {
            class: "tag-input-container",
            {selected_snapshot.iter().map(|tag| {
                let tag_for_closure = tag.clone();
                let (bg, fg, border) = tag_colors(tag);
                let style = format!("background: {bg}; color: {fg}; border: 1px solid {border};");
                rsx! {
                    span {
                        class: "tag-pill",
                        key: "{tag}",
                        style: "{style}",
                        span { "{tag}" },
                        span {
                            class: "remove-tag",
                            onclick: move |_| {
                                selected_tags.write().retain(|t| t != &tag_for_closure);
                                props.on_change.call(selected_tags.read().clone());
                            },
                            "×"
                        }
                    }
                }
            })}
            input {
                r#type: "text",
                class: "tag-input",
                placeholder: "新增標籤...",
                value: "{current_input}",
                oninput: move |evt| current_input.set(evt.value()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter || evt.key() == Key::Character(",".to_string()) {
                        evt.prevent_default();
                        let raw = current_input.read().trim().to_string();
                        if let Some(new_tag) = normalize_category_label(&raw) {
                            let should_add = !normalized_selected.contains(&new_tag);
                            if should_add {
                                selected_tags.write().push(new_tag);
                                props.on_change.call(selected_tags.read().clone());
                            }
                            current_input.set("".to_string());
                        }
                    }
                    if evt.key() == Key::Backspace && current_input.read().is_empty() {
                        evt.prevent_default();
                        if selected_tags.write().pop().is_some() {
                            props.on_change.call(selected_tags.read().clone());
                        }
                    }
                }
            }
        }
        if !available_suggestions.is_empty() {
            div {
                class: "tag-suggestions",
                span { class: "tag-suggestions-label", "常用標籤：" }
                {available_suggestions.clone().into_iter().map(|tag| {
                    let (bg, fg, border) = tag_colors(&tag);
                    let style =
                        format!("background: {bg}; color: {fg}; border: 1px solid {border};");
                    let tag_for_closure = tag.clone();
                    rsx!(
                        button {
                            r#type: "button",
                            class: "tag-suggestion-pill",
                            style: "{style}",
                            onclick: move |_| {
                                let mut next = selected_tags.read().clone();
                                if !next.contains(&tag_for_closure) {
                                    next.push(tag_for_closure.clone());
                                    selected_tags.set(next.clone());
                                    props.on_change.call(next);
                                }
                            },
                            "{tag}"
                        }
                    )
                })}
            }
        }
    }
}
