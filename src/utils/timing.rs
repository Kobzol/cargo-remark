use std::time::Instant;

pub fn time_block<F: FnOnce() -> R, R>(label: &str, f: F) -> R {
    let start = Instant::now();
    let result = f();
    log::debug!("{label} ({:.2}s)", start.elapsed().as_secs_f32());
    result
}
