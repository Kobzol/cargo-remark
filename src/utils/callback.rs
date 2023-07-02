pub trait LoadCallback {
    fn start(&self, count: u64);
    fn advance(&self);
    fn finish(&self);
}
