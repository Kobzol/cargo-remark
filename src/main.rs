use crate::remark::index::RemarkIndex;
use env_logger::Env;
use std::path::Path;

use crate::remark::load_remarks_from_dir;
use crate::render::render_index;

mod remark;
mod render;
mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("cargo_remark=info")).init();

    let file = "remarks";
    let remarks = load_remarks_from_dir(file)?;
    let index = RemarkIndex::new(remarks);
    render_index(index, Path::new("out"))?;

    Ok(())
}
