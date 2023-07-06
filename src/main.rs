mod cargo;

use crate::cargo::build;
use crate::cargo::cli::cli_format_path;
use crate::cargo::version::check_remark_dir_support;
use cargo_remark::render::INDEX_FILE_PATH;
use clap::Parser;
use env_logger::Env;

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
    /// Build a crate with optimizations, generate optimization remarks and display a website
    /// with remark summary.
    Build(BuildArgs),
}

#[derive(clap::Parser, Debug)]
struct BuildArgs {
    /// Open the generated website after the build finishes.
    #[arg(long)]
    open: bool,

    /// Additional arguments that will be passed to `cargo build`.
    cargo_args: Vec<String>,
}

fn command_build(args: BuildArgs) -> anyhow::Result<()> {
    if !check_remark_dir_support()? {
        return Err(anyhow::anyhow!(
            "Your version of rustc does not support `-Zremark-dir`. Please use a nightly version not older than 4. 7. 2023."
        ));
    }
    let output = build(args.cargo_args)?;
    let index_path = output.out_dir.join(INDEX_FILE_PATH);

    if args.open {
        opener::open_browser(&index_path).map_err(|error| {
            anyhow::anyhow!(
                "Could not open {} in browser: {error:?}",
                cli_format_path(index_path)
            )
        })?;
    } else {
        log::info!(
            "Open {} in a browser to see the results.",
            cli_format_path(index_path)
        );
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("cargo_remark=info")).init();

    let args = Args::parse();

    match args {
        Args::Remark(args) => match args {
            Subcommand::Build(args) => command_build(args),
        },
    }
}
