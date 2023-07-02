use std::time::Instant;

pub fn time_block_log<F: FnOnce() -> R, R>(label: &str, f: F) -> R {
    let start = Instant::now();
    let result = f();
    log::debug!("{label} ({:.2}s)", start.elapsed().as_secs_f32());
    result
}

pub fn time_block_print<F: FnOnce() -> R, R>(label: &str, f: F) -> R {
    let start = Instant::now();
    let result = f();
    eprintln!("{label} finished in {:.2}s", start.elapsed().as_secs_f32());
    result
}
