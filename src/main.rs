use std::path::PathBuf;

use clap::Parser;
use env_logger::Env;
use indicatif::ProgressBar;

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
    /// Build Cargo crate with optimizations, generate optimization remarks and display a website
    /// that analyzes the results.
    Build(BuildArgs),
    /// Analyze a directory with LLVM remarks.
    Analyze(AnalyzeArgs),
}

#[derive(clap::Parser, Debug)]
struct BuildArgs {
    /// Additional arguments that will be passed to `cargo build`.
    cargo_args: Vec<String>,
}

#[derive(clap::Parser, Debug)]
struct AnalyzeArgs {
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
    time_block_print("Render", || {
        render_remarks(
            remarks,
            &args.source_dir,
            &args.output_dir,
            Some(&ProgressBarCallback::new()),
        )
    })?;
    Ok(())
}

fn command_build(_args: BuildArgs) -> anyhow::Result<()> {
    // TODO: waiting for `-Zremark-dir` to be merged...
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("cargo_remark=info")).init();

    let args = Args::parse();

    match args {
        Args::Remark(args) => match args {
            Subcommand::Analyze(args) => command_analyze(args),
            Subcommand::Build(args) => command_build(args),
        },
    }
}
