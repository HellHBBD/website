use dioxus::prelude::*;
use crate::category::normalize_category_label;
use crate::task::Task;

#[cfg(not(target_arch = "wasm32"))]
fn storage_file() -> String {
    std::env::var("TASKS_STORAGE_FILE").unwrap_or_else(|_| "tasks.json".to_string())
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
async fn write_tasks_to_disk(tasks: &[Task]) -> Result<(), ServerFnError> {
    use tokio::fs;

    let json = serde_json::to_string(tasks)
        .map_err(|e| ServerFnError::new(format!("Failed to serialize JSON: {}", e)))?;

    fs::write(storage_file(), json)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to save JSON: {}", e)))?;

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
        Err(e) => {
            return Err(ServerFnError::new(format!(
                "Failed to read tasks file: {}",
                e
            )));
        }
    };

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&content)
        .map_err(|e| ServerFnError::new(format!("Failed to parse JSON: {}", e)))
}

/// Website -> Server
#[server(prefix = "/api", endpoint = "upload")]
pub async fn upload(data: Vec<Task>) -> Result<(), ServerFnError> {
    info!("Uploading tasks");
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut data = data;
        normalize_tasks(&mut data);
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
    let result = read_tasks_from_disk().await.map(|mut tasks| {
        normalize_tasks(&mut tasks);
        tasks
    });
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
    info!(
        "[{}] --- Starting check_and_update_tasks ---",
        Local::now()
    );
    #[cfg(not(target_arch = "wasm32"))]
    let mut tasks = read_tasks_from_disk().await?;
    #[cfg(not(target_arch = "wasm32"))]
    normalize_tasks(&mut tasks);
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

    info!(
        "[{}] --- Finished check_and_update_tasks ---",
        Local::now()
    );
    Ok(tasks)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate};
    use crate::test_support::{
        env_lock, restore_storage_file, set_storage_file, temp_storage_path,
    };
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

        tokio::fs::write(&path, "")
            .await
            .expect("write empty file");

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
