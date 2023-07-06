mod cargo;

use crate::cargo::build;
use crate::cargo::version::check_remark_dir_support;
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
    /// Build Cargo crate with optimizations, generate optimization remarks and display a website
    /// that analyzes the results.
    Build(BuildArgs),
}

#[derive(clap::Parser, Debug)]
struct BuildArgs {
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
