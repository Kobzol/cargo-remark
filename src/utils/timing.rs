use std::time::Instant;

pub fn time_block<F: FnOnce()>(label: &str, f: F) {
    let start = Instant::now();
    f();
    log::debug!("{label} ({:.2}s)", start.elapsed().as_secs_f32());
}
