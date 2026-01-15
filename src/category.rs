use std::collections::BTreeSet;

pub const UNCATEGORIZED_LABEL: &str = "未分類";

pub fn normalize_category_label(raw: &str) -> Option<String> {
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() || normalized == UNCATEGORIZED_LABEL {
        None
    } else {
        Some(normalized)
    }
}

pub fn normalized_set(values: &[String]) -> BTreeSet<String> {
    values
        .iter()
        .filter_map(|value| normalize_category_label(value))
        .collect()
}

pub fn collect_available_categories<'a, I>(values: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = &'a String>,
{
    values
        .into_iter()
        .filter_map(|value| normalize_category_label(value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_category_label_trims_and_collapses_space() {
        assert_eq!(
            normalize_category_label("  foo   bar  "),
            Some("foo bar".to_string())
        );
        assert_eq!(
            normalize_category_label("single"),
            Some("single".to_string())
        );
    }

    #[test]
    fn test_normalize_category_label_filters_uncategorized() {
        assert_eq!(normalize_category_label(UNCATEGORIZED_LABEL), None);
        assert_eq!(
            normalize_category_label(&format!("   {UNCATEGORIZED_LABEL}   ")),
            None
        );
        assert_eq!(normalize_category_label("   "), None);
    }

    #[test]
    fn test_normalized_set_dedupes_and_filters() {
        let values = vec![
            "work".to_string(),
            "  work  ".to_string(),
            "home".to_string(),
            UNCATEGORIZED_LABEL.to_string(),
        ];
        let set = normalized_set(&values);
        assert!(set.contains("work"));
        assert!(set.contains("home"));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_collect_available_categories_filters_uncategorized() {
        let values = vec![
            "work".to_string(),
            UNCATEGORIZED_LABEL.to_string(),
            "home  task".to_string(),
        ];
        let set = collect_available_categories(values.iter());
        assert!(set.contains("work"));
        assert!(set.contains("home task"));
        assert_eq!(set.len(), 2);
    }
}
