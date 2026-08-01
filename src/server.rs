use crate::category::normalize_category_label;
use crate::task::Task;
use dioxus::prelude::*;

const MAX_TASKS: usize = 500;
const MAX_NAME_LENGTH: usize = 200;
const MAX_CATEGORIES_PER_TASK: usize = 32;
const MAX_CATEGORY_LENGTH: usize = 64;
const MAX_INTERVAL_DAYS: u32 = 3_650;
const MAX_COMPLETED_COUNT: u32 = 1_000_000;
const MIN_GROWTH_RATE: f64 = 0.1;
const MAX_GROWTH_RATE: f64 = 10.0;

#[cfg(not(target_arch = "wasm32"))]
fn storage_file() -> String {
    std::env::var("TASKS_STORAGE_FILE").unwrap_or_else(|_| "tasks.json".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn task_storage_lock() -> &'static tokio::sync::Mutex<()> {
    use std::sync::OnceLock;

    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_tasks(tasks: &mut [Task]) {
    use chrono::Local;

    for task in tasks {
        if task.created_at.is_none() {
            task.created_at = Some(Local::now().naive_local());
        }
        let mut normalized = Vec::new();
        for category in &task.categories {
            if let Some(value) = normalize_category_label(category) {
                normalized.push(value);
            }
        }
        task.categories = normalized;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_tasks(tasks: &[Task]) -> Result<(), ServerFnError> {
    use std::collections::HashSet;

    if tasks.len() > MAX_TASKS {
        return Err(ServerFnError::new("Invalid task data"));
    }

    let mut ids = HashSet::with_capacity(tasks.len());
    for task in tasks {
        if task.name.trim().is_empty()
            || task.name.chars().count() > MAX_NAME_LENGTH
            || task.categories.len() > MAX_CATEGORIES_PER_TASK
            || task.categories.iter().any(|category| {
                category.chars().count() > MAX_CATEGORY_LENGTH || category.trim().is_empty()
            })
            || task.interval == 0
            || task.interval > MAX_INTERVAL_DAYS
            || task
                .date
                .checked_add_signed(chrono::Duration::days(i64::from(task.interval)))
                .is_none()
            || !task.growth_rate.is_finite()
            || !(MIN_GROWTH_RATE..=MAX_GROWTH_RATE).contains(&task.growth_rate)
            || task.completed_count > MAX_COMPLETED_COUNT
            || !ids.insert(task.id)
        {
            return Err(ServerFnError::new("Invalid task data"));
        }
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn write_tasks_to_disk(tasks: &[Task]) -> Result<(), ServerFnError> {
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    let json = serde_json::to_string(tasks).map_err(|error| {
        error!("Failed to serialize task data: {error}");
        ServerFnError::new("Unable to save task data")
    })?;
    let storage_file = storage_file();
    let temporary_file = format!("{storage_file}.tmp");
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options.open(&temporary_file).await.map_err(|error| {
        error!("Failed to open temporary task storage file: {error}");
        ServerFnError::new("Unable to save task data")
    })?;
    file.write_all(json.as_bytes()).await.map_err(|error| {
        error!("Failed to write temporary task storage file: {error}");
        ServerFnError::new("Unable to save task data")
    })?;
    file.sync_all().await.map_err(|error| {
        error!("Failed to sync temporary task storage file: {error}");
        ServerFnError::new("Unable to save task data")
    })?;
    drop(file);

    fs::rename(&temporary_file, &storage_file)
        .await
        .map_err(|error| {
            error!("Failed to replace task storage file: {error}");
            ServerFnError::new("Unable to save task data")
        })?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_tasks_from_disk() -> Result<Vec<Task>, ServerFnError> {
    use tokio::fs;

    let content = match fs::read_to_string(storage_file()).await {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => {
            error!("Failed to read task storage file: {error}");
            return Err(ServerFnError::new("Unable to read task data"));
        }
    };

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&content).map_err(|error| {
        error!("Failed to parse task storage JSON: {error}");
        ServerFnError::new("Unable to read task data")
    })
}

/// Website -> Server
#[server(prefix = "/api", endpoint = "upload")]
pub async fn upload(data: Vec<Task>) -> Result<(), ServerFnError> {
    info!("Uploading tasks");
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _guard = task_storage_lock().lock().await;
        let mut data = data;
        normalize_tasks(&mut data);
        validate_tasks(&data)?;
        write_tasks_to_disk(&data).await?;
    }
    Ok(())
}

/// Server -> Website
#[server(prefix = "/api", endpoint = "download")]
pub async fn download() -> Result<Vec<Task>, ServerFnError> {
    use chrono::Local;
    info!("[{}] --- Starting download ---", Local::now());
    #[cfg(not(target_arch = "wasm32"))]
    let result = {
        let _guard = task_storage_lock().lock().await;
        read_tasks_from_disk().await.and_then(|mut tasks| {
            normalize_tasks(&mut tasks);
            validate_tasks(&tasks)?;
            Ok(tasks)
        })
    };
    #[cfg(not(target_arch = "wasm32"))]
    info!("[{}] --- Finished download ---", Local::now());
    #[cfg(not(target_arch = "wasm32"))]
    return result;
    #[cfg(target_arch = "wasm32")]
    Ok(Vec::new())
}

#[server(prefix = "/api", endpoint = "check_and_update_tasks")]
pub async fn check_and_update_tasks() -> Result<Vec<Task>, ServerFnError> {
    use chrono::Local;
    info!("[{}] --- Starting check_and_update_tasks ---", Local::now());
    #[cfg(not(target_arch = "wasm32"))]
    let _guard = task_storage_lock().lock().await;
    #[cfg(not(target_arch = "wasm32"))]
    let mut tasks = read_tasks_from_disk().await?;
    #[cfg(not(target_arch = "wasm32"))]
    normalize_tasks(&mut tasks);
    #[cfg(not(target_arch = "wasm32"))]
    validate_tasks(&tasks)?;
    #[cfg(target_arch = "wasm32")]
    let mut tasks: Vec<Task> = Vec::new();

    let today = Local::now().date_naive();
    let mut modified = false;
    for task in &mut tasks {
        if task.date < today {
            task.date = today;
            modified = true;
        }
    }

    if modified {
        info!(
            "[{}] --- check_and_update_tasks: Tasks modified, starting upload ---",
            Local::now()
        );
        #[cfg(not(target_arch = "wasm32"))]
        write_tasks_to_disk(&tasks).await?;
        info!(
            "[{}] --- check_and_update_tasks: Finished upload ---",
            Local::now()
        );
    }

    info!("[{}] --- Finished check_and_update_tasks ---", Local::now());
    Ok(tasks)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::test_support::{
        env_lock, restore_storage_file, set_storage_file, temp_storage_path,
    };
    use chrono::{Duration, NaiveDate};
    use uuid::Uuid;

    fn sample_task(date: NaiveDate) -> Task {
        Task {
            id: Uuid::new_v4(),
            name: "Sample".to_string(),
            date,
            interval: 3,
            growth_rate: 1.2,
            completed_count: 0,
            created_at: None,
            categories: Vec::new(),
        }
    }

    #[test]
    fn validate_tasks_rejects_unsafe_values() {
        let today = chrono::Local::now().date_naive();
        let mut task = sample_task(today);
        task.name = " ".to_string();
        assert!(validate_tasks(&[task]).is_err());

        let mut task = sample_task(today);
        task.growth_rate = f64::INFINITY;
        assert!(validate_tasks(&[task]).is_err());

        let task = sample_task(today);
        assert!(validate_tasks(&[task.clone(), task]).is_err());

        let mut task = sample_task(chrono::NaiveDate::MAX);
        task.interval = 1;
        assert!(validate_tasks(&[task]).is_err());
    }

    #[tokio::test]
    async fn read_tasks_missing_file_returns_empty() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_storage_path();
        let previous = set_storage_file(&path);

        let result = read_tasks_from_disk().await.expect("read ok");
        restore_storage_file(previous);

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn read_tasks_empty_file_returns_empty() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_storage_path();
        let previous = set_storage_file(&path);

        tokio::fs::write(&path, "").await.expect("write empty file");

        let result = read_tasks_from_disk().await.expect("read ok");
        restore_storage_file(previous);

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn write_then_read_roundtrip() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_storage_path();
        let previous = set_storage_file(&path);
        let today = chrono::Local::now().date_naive();
        let tasks = vec![sample_task(today)];

        write_tasks_to_disk(&tasks).await.expect("write ok");
        let loaded = read_tasks_from_disk().await.expect("read ok");
        restore_storage_file(previous);

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Sample");
        assert_eq!(loaded[0].date, today);
    }

    #[tokio::test]
    async fn check_and_update_tasks_moves_past_dates() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_storage_path();
        let previous = set_storage_file(&path);
        let today = chrono::Local::now().date_naive();
        let tasks = vec![sample_task(today - Duration::days(1))];

        write_tasks_to_disk(&tasks).await.expect("write ok");
        let updated = check_and_update_tasks().await.expect("update ok");
        restore_storage_file(previous);

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].date, today);
    }
}
