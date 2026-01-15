use chrono::{Datelike, Local};
use dioxus::prelude::*;

static HISTOGRAM_IMAGE: Asset = asset!("assets/img.png");

#[component]
pub fn HistogramPage() -> Element {
    let mut input_value = use_signal(String::new);
    rsx! {
        main { class: "home",
            div { class: "home-bg" }
            section { class: "histogram-shell histogram-panel",
                div { class: "histogram-card",
                    img {
                        class: "histogram-figure",
                        src: HISTOGRAM_IMAGE
                    }
                    div { class: "histogram-controls",
                        input {
                            class: "histogram-input",
                            r#type: "text",
                            placeholder: "輸入課程代碼...",
                            value: "{input_value}",
                            oninput: move |evt| {
                                let sanitized = sanitize_course_code(evt.value());
                                input_value.set(sanitized);
                            },
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter {
                                    evt.prevent_default();
                                    let value = input_value.read();
                                    if is_valid_course_code(&value) {
                                        let url = build_histogram_url(&value);
                                        open_histogram_url(&url);
                                    }
                                }
                            }
                        }
                        button {
                            class: "histogram-button",
                            r#type: "button",
                            onclick: move |_| {
                                let value = input_value.read();
                                if is_valid_course_code(&value) {
                                    let url = build_histogram_url(&value);
                                    open_histogram_url(&url);
                                }
                            },
                            "查看成績分佈"
                        }
                    }
                }
            }
        }
    }
}

fn sanitize_course_code(input: String) -> String {
    let trimmed = input.trim();
    let mut sanitized = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_uppercase());
        }
    }
    sanitized
}

fn is_valid_course_code(input: &str) -> bool {
    !input.is_empty() && input.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn build_histogram_url(input: &str) -> String {
    let now = Local::now().naive_local().date();
    let roc_year = now.year() - 1911;
    let syear = roc_year.saturating_sub(1);
    let sem = if now.month() >= 7 { 2 } else { 1 };
    let trimmed = input.trim();
    let mut chars = trimmed.chars();
    let co_no: String = chars.by_ref().take(7).collect();
    let class_code: String = chars.collect();
    let syear = format!("{:04}", syear);
    format!(
        "https://qrys.ncku.edu.tw/ncku/histogram.asp?syear={syear}&sem={sem}&co_no={co_no}&class_code={class_code}"
    )
}

#[cfg(target_arch = "wasm32")]
fn open_histogram_url(url: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(url);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_histogram_url(_url: &str) {}
