use chrono::{Local, NaiveDate, NaiveDateTime};
use dioxus::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;
use crate::category::normalize_category_label;

#[derive(Clone, PartialEq, Props, Debug, Deserialize, Serialize)]
pub struct Task {
    #[serde(default = "default_id")]
    pub id: Uuid,
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_date")]
    pub date: NaiveDate,
    #[serde(default = "default_interval")]
    pub interval: u32,
    #[serde(default = "default_growth_rate")]
    pub growth_rate: f64,
    #[serde(default = "default_completed_count")]
    pub completed_count: u32,
    #[serde(default = "default_created_at")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(
        default,
        alias = "category",
        deserialize_with = "deserialize_categories"
    )]
    pub categories: Vec<String>,
}

fn default_id() -> Uuid {
    Uuid::new_v4()
}

fn default_name() -> String {
    String::new()
}

fn default_date() -> NaiveDate {
    Local::now().date_naive()
}

fn default_interval() -> u32 {
    2
}

fn default_growth_rate() -> f64 {
    1.3
}

fn default_completed_count() -> u32 {
    0
}

fn default_created_at() -> Option<NaiveDateTime> {
    None
}

fn deserialize_categories<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Categories {
        One(String),
        Many(Vec<String>),
    }

    let categories = Categories::deserialize(deserializer)?;
    let values = match categories {
        Categories::One(value) => vec![value],
        Categories::Many(values) => values,
    };

    Ok(values
        .into_iter()
        .filter_map(|value| normalize_category_label(&value))
        .collect())
}

pub fn insert_task_sorted(tasks: &mut Vec<Task>, task: Task) {
    let fallback_created_at = NaiveDate::from_ymd_opt(1970, 1, 1)
        .expect("valid date")
        .and_hms_opt(0, 0, 0)
        .expect("valid time");
    let new_index = tasks
        .binary_search_by(|t| {
            let lhs = t.created_at.unwrap_or(fallback_created_at);
            let rhs = task.created_at.unwrap_or(fallback_created_at);
            rhs.cmp(&lhs)
                .then_with(|| {
                    t.date
                        .cmp(&task.date)
                        .then_with(|| task.completed_count.cmp(&t.completed_count))
                })
        })
        .unwrap_or_else(|e| e);

    tasks.insert(new_index, task);
}

pub fn upsert_task(tasks: &mut Vec<Task>, task: Task) {
    if let Some(index) = tasks.iter().position(|t| t.id == task.id) {
        tasks.remove(index);
    }

    insert_task_sorted(tasks, task);
}

pub fn complete_task(tasks: &mut Vec<Task>, task_id: Uuid, today: NaiveDate) -> bool {
    let Some(index) = tasks.iter().position(|t| t.id == task_id) else {
        return false;
    };

    let mut task = tasks.remove(index);
    task.completed_count += 1;

    if task.date < today {
        task.date = today + chrono::Duration::days(task.interval as i64);
    } else {
        task.date += chrono::Duration::days(task.interval as i64);
    }

    task.interval = (task.interval as f64 * task.growth_rate).round() as u32;
    insert_task_sorted(tasks, task);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::UNCATEGORIZED_LABEL;
    use chrono::Local;
    use serde::Deserialize;

    #[test]
    fn test_task_creation() {
        let id = Uuid::new_v4();
        let name = "Test Task".to_string();
        let date = Local::now().date_naive();
        let interval = 7;
        let growth_rate = 1.5;
        let completed_count = 0;
        let created_at = Some(Local::now().naive_local());
        let categories = Vec::new();

        let task = Task {
            id,
            name: name.clone(),
            date,
            interval,
            growth_rate,
            completed_count,
            created_at,
            categories: categories.clone(),
        };

        assert_eq!(task.id, id);
        assert_eq!(task.name, name);
        assert_eq!(task.date, date);
        assert_eq!(task.interval, interval);
        assert_eq!(task.growth_rate, growth_rate);
        assert_eq!(task.completed_count, completed_count);
        assert_eq!(task.created_at, created_at);
        assert_eq!(task.categories, categories);
    }

    #[test]
    fn test_insert_task_sorted() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");
        let mut tasks = Vec::new();

        insert_task_sorted(
            &mut tasks,
            Task {
                id: Uuid::new_v4(),
                name: "Second".to_string(),
                date: date.succ_opt().expect("next day"),
                interval: 1,
                growth_rate: 1.0,
                completed_count: 0,
                created_at: None,
                categories: Vec::new(),
            },
        );

        insert_task_sorted(
            &mut tasks,
            Task {
                id: Uuid::new_v4(),
                name: "First".to_string(),
                date,
                interval: 1,
                growth_rate: 1.0,
                completed_count: 0,
                created_at: None,
                categories: Vec::new(),
            },
        );

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "First");
        assert_eq!(tasks[1].name, "Second");
    }

    #[test]
    fn test_complete_task_updates_order() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");
        let mut tasks = vec![
            Task {
                id: Uuid::new_v4(),
                name: "A".to_string(),
                date: today,
                interval: 2,
                growth_rate: 1.0,
                completed_count: 0,
                created_at: None,
                categories: Vec::new(),
            },
            Task {
                id: Uuid::new_v4(),
                name: "B".to_string(),
                date: today.succ_opt().expect("next day"),
                interval: 2,
                growth_rate: 1.0,
                completed_count: 0,
                created_at: None,
                categories: Vec::new(),
            },
        ];

        let first_id = tasks[0].id;
        let completed = complete_task(&mut tasks, first_id, today);

        assert!(completed);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "B");
        assert_eq!(tasks[1].completed_count, 1);
    }

    #[test]
    fn test_upsert_task_replaces_and_sorts() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");
        let tomorrow = today.succ_opt().expect("next day");
        let first_id = Uuid::new_v4();
        let second_id = Uuid::new_v4();

        let mut tasks = vec![
            Task {
                id: first_id,
                name: "First".to_string(),
                date: tomorrow,
                interval: 1,
                growth_rate: 1.0,
                completed_count: 0,
                created_at: None,
                categories: Vec::new(),
            },
            Task {
                id: second_id,
                name: "Second".to_string(),
                date: today,
                interval: 1,
                growth_rate: 1.0,
                completed_count: 0,
                created_at: None,
                categories: Vec::new(),
            },
        ];

        let replacement = Task {
            id: first_id,
            name: "First Updated".to_string(),
            date: today,
            interval: 2,
            growth_rate: 1.0,
            completed_count: 1,
            created_at: None,
            categories: Vec::new(),
        };

        upsert_task(&mut tasks, replacement);

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "First Updated");
        assert_eq!(tasks[0].completed_count, 1);
        assert_eq!(tasks[1].name, "Second");
    }

    #[test]
    fn test_complete_task_missing_returns_false() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date");
        let mut tasks = Vec::new();

        let completed = complete_task(&mut tasks, Uuid::new_v4(), today);

        assert!(!completed);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_deserialize_categories_accepts_string() {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "deserialize_categories")]
            categories: Vec<String>,
        }

        let value: Wrapper =
            serde_json::from_str(r#"{ "categories": "  foo   bar  " }"#).expect("valid json");
        assert_eq!(value.categories, vec!["foo bar".to_string()]);
    }

    #[test]
    fn test_deserialize_categories_filters_uncategorized() {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "deserialize_categories")]
            categories: Vec<String>,
        }

        let value: Wrapper = serde_json::from_str(
            &format!(
                r#"{{
                    "categories": ["work", "{UNCATEGORIZED_LABEL}", "  ", "home  task"]
                }}"#
            ),
        )
        .expect("valid json");
        assert_eq!(
            value.categories,
            vec!["work".to_string(), "home task".to_string()]
        );
    }
}
