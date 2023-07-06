use cargo_remark::remark::{load_remarks_from_dir, RemarkLoadOptions};
use cargo_remark::render::render_remarks;
use cargo_remark::utils::callback::ProgressBarCallback;
use cargo_remark::utils::timing::time_block_print;
use clap::Parser;
use env_logger::Env;
use std::path::PathBuf;

/// Analyze a directory containing YAML files with LLVM optimization remarks
#[derive(clap::Parser, Debug)]
struct Args {
    /// Directory containing remark files in YAML format.
    /// They have to end with the `.opt.yaml` extension.
    #[arg()]
    remark_dir: PathBuf,

    /// Root directory of source (crate) from which the remarks were generated.
    #[arg(long)]
    source_dir: PathBuf,

    /// Output directory into which a HTML website with remark information will be generated.
    #[arg(long, default_value = "out")]
    output_dir: PathBuf,

    /// Load remarks from external code (i.e. crat edependencies).
    /// Note that this may produce a large amount of data!
    #[arg(long)]
    external: bool,
}

fn analyze(args: Args) -> anyhow::Result<()> {
    let Args {
        remark_dir,
        source_dir,
        output_dir,
        external,
    } = args;

    let remarks = time_block_print("Remark loading", || {
        load_remarks_from_dir(
            remark_dir,
            RemarkLoadOptions { external },
            Some(&ProgressBarCallback::default()),
        )
    })?;
    time_block_print("Render", || {
        render_remarks(
            remarks,
            &source_dir,
            &output_dir,
            Some(&ProgressBarCallback::default()),
        )
    })?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("cargo_remark=info")).init();

    let args = Args::parse();
    analyze(args)?;
    Ok(())
}
