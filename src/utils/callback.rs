use indicatif::ProgressBar;

pub trait LoadCallback {
    fn start(&self, count: u64);
    fn advance(&self);
    fn finish(&self);
}

pub struct ProgressBarCallback {
    pbar: ProgressBar,
}

impl Default for ProgressBarCallback {
    fn default() -> Self {
        Self {
            pbar: ProgressBar::new(1),
        }
    }
}

impl LoadCallback for ProgressBarCallback {
    fn start(&self, count: u64) {
        self.pbar.set_length(count);
    }

    fn advance(&self) {
        self.pbar.inc(1);
    }

    fn finish(&self) {
        self.pbar.finish_using_style();
    }
}
