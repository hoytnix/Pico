use std::collections::BTreeMap;

/// Chunks a slice of items into pages of `page_size` (e.g. 10).
/// Returns 0 chunks for 0 items, 1 chunk for 1..=10 items, 2 chunks for 11..=20 items, etc.
pub fn paginate_chunks<T: Clone>(items: &[T], page_size: usize) -> Vec<Vec<T>> {
    let step = page_size.max(1);
    if items.is_empty() {
        return Vec::new();
    }
    items.chunks(step).map(|c| c.to_vec()).collect()
}

/// Groups items by tag in deterministic sorted order.
pub fn group_by_tag<T: Clone, F: Fn(&T) -> &[String]>(
    items: &[T],
    get_tags: F,
) -> BTreeMap<String, Vec<T>> {
    let mut grouped: BTreeMap<String, Vec<T>> = BTreeMap::new();
    for item in items {
        for tag in get_tags(item) {
            let trimmed = tag.trim();
            if !trimmed.is_empty() {
                grouped
                    .entry(trimmed.to_string())
                    .or_default()
                    .push(item.clone());
            }
        }
    }
    grouped
}
