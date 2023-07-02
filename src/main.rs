use std::path::PathBuf;

use clap::Parser;
use env_logger::Env;
use indicatif::ProgressBar;

use cargo_remark::remark::index::RemarkIndex;
use cargo_remark::remark::load_remarks_from_dir;
use cargo_remark::render::render_remarks;
use cargo_remark::utils::callback::LoadCallback;
use cargo_remark::utils::timing::time_block_print;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about)]
#[clap(bin_name("cargo"))]
#[clap(disable_help_subcommand(true))]
enum Args {
    #[clap(subcommand)]
    #[clap(author, version, about)]
    Remark(Subcommand),
}

#[derive(clap::Subcommand, Debug)]
enum Subcommand {
    /// Analyze a directory with LLVM remarks.
    Analyze(AnalyzeArgs),
}

#[derive(clap::Parser, Debug)]
struct AnalyzeArgs {
    #[arg()]
    remark_dir: PathBuf,

    #[arg(long, default_value = "out")]
    output_dir: PathBuf,
}

struct ProgressBarCallback {
    pbar: ProgressBar,
}

impl ProgressBarCallback {
    fn new() -> Self {
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

fn command_analyze(args: AnalyzeArgs) -> anyhow::Result<()> {
    let remarks = time_block_print("Remark loading", || {
        load_remarks_from_dir(args.remark_dir, Some(&ProgressBarCallback::new()))
    })?;
    let index = time_block_print("Index load", || RemarkIndex::new(remarks));
    time_block_print("Render", || render_remarks(index, &args.output_dir))?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("cargo_remark=info")).init();

    let args = Args::parse();

    match args {
        Args::Remark(args) => match args {
            Subcommand::Analyze(args) => command_analyze(args),
        },
    }
}
