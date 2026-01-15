use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

pub fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn temp_storage_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("schedule-test-{}.json", Uuid::new_v4()));
    path
}

pub fn set_storage_file(path: &Path) -> Option<String> {
    let previous = std::env::var("TASKS_STORAGE_FILE").ok();
    std::env::set_var("TASKS_STORAGE_FILE", path);
    previous
}

pub fn restore_storage_file(previous: Option<String>) {
    if let Some(value) = previous {
        std::env::set_var("TASKS_STORAGE_FILE", value);
    } else {
        std::env::remove_var("TASKS_STORAGE_FILE");
    }
}

pub fn set_public_path(path: &Path) -> Option<String> {
    let previous = std::env::var("DIOXUS_PUBLIC_PATH").ok();
    std::env::set_var("DIOXUS_PUBLIC_PATH", path);
    previous
}

pub fn restore_public_path(previous: Option<String>) {
    if let Some(value) = previous {
        std::env::set_var("DIOXUS_PUBLIC_PATH", value);
    } else {
        std::env::remove_var("DIOXUS_PUBLIC_PATH");
    }
}

pub fn temp_public_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("schedule-public-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&path).expect("create public dir");
    let assets_dir = path.join("assets");
    std::fs::create_dir_all(&assets_dir).expect("create assets dir");
    std::fs::write(
        path.join("index.html"),
        "<!DOCTYPE html>\n<html>\n    <head> </head>\n    <body>\n        <div id=\"main\"></div>\n    </body>\n</html>\n",
    )
    .expect("write index.html");
    std::fs::write(assets_dir.join("app.js"), "console.log('ok');\n")
        .expect("write app.js");
    std::fs::write(assets_dir.join("app.wasm"), [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00])
        .expect("write app.wasm");
    path
}
