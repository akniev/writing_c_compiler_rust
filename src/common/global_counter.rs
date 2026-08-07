use std::sync::atomic::{AtomicI32, Ordering};

static GLOBAL_COUNTER: AtomicI32 = AtomicI32::new(0);

pub fn get_global_counter() -> i32 {
    GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn get_unique_label(prefix: &str) -> String {
    let counter = get_global_counter();
    format!("{}.{}", prefix, counter)
}