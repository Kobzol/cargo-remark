use env_logger::Env;

use crate::remark::load_remarks_from_dir;

mod remark;
mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("cargo_remark=info")).init();

    let file = "remarks";
    let remarks = load_remarks_from_dir(file)?;
    dbg!(remarks);

    Ok(())
}
