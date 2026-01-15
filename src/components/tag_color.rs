use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static TAG_COLOR_CACHE: RefCell<HashMap<String, (String, String, String)>> =
        RefCell::new(HashMap::new());
}

pub const TAG_COLOR_CACHE_LIMIT: usize = 512; // Prevent unbounded growth across long sessions.

pub fn tag_colors(label: &str) -> (String, String, String) {
    if let Some(cached) = TAG_COLOR_CACHE.with(|cache| cache.borrow().get(label).cloned()) {
        return cached;
    }

    let mut hash: u32 = 0x811c9dc5;
    for b in label.as_bytes() {
        hash ^= u32::from(*b);
        hash = hash.wrapping_mul(0x01000193);
    }

    let mut seed = hash;
    let mut next = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed as f64) / (u32::MAX as f64)
    };

    let h = (next() * 360.0).floor() as u32;
    let s = (55.0 + next() * 20.0).floor() as u32;
    let l_bg = (22.0 + next() * 8.0).floor() as u32;
    let l_fg = (88.0 + next() * 6.0).floor() as u32;
    let s_fg = (s + 10).min(85);
    let l_border = (l_bg + 10).min(40);

    let bg = format!("hsl({h} {s}% {l_bg}%)");
    let fg = format!("hsl({h} {s_fg}% {l_fg}%)");
    let border = format!("hsl({h} {s}% {l_border}%)");
    let colors = (bg, fg, border);
    TAG_COLOR_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() >= TAG_COLOR_CACHE_LIMIT {
            cache.clear();
        }
        cache.insert(label.to_string(), colors.clone());
    });
    colors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_colors_stable_per_label() {
        let first = tag_colors("alpha");
        let second = tag_colors("alpha");
        assert_eq!(first, second);
    }

    #[test]
    fn test_tag_colors_stable_after_cache_clear() {
        let expected = tag_colors("stable");
        for index in 0..(TAG_COLOR_CACHE_LIMIT + 10) {
            let label = format!("label-{index}");
            let _ = tag_colors(&label);
        }
        let actual = tag_colors("stable");
        assert_eq!(expected, actual);
    }
}
