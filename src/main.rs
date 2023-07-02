use std::path::PathBuf;

use clap::Parser;
use env_logger::Env;

use cargo_remark::remark::index::RemarkIndex;
use cargo_remark::remark::load_remarks_from_dir;
use cargo_remark::render::render_index;

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

fn command_analyze(args: AnalyzeArgs) -> anyhow::Result<()> {
    let remarks = load_remarks_from_dir(args.remark_dir)?;
    let index = RemarkIndex::new(remarks);
    render_index(index, &args.output_dir)?;
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
