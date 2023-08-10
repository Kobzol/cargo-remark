mod cargo;

use cargo::cli::cli_format_path;
use cargo::version::check_remark_dir_support;
use cargo::{get_rustc_source_root, run_cargo, CargoSubcommand};
use cargo_remark::remark::{load_remarks_from_dir, RemarkLoadOptions};
use cargo_remark::render::{render_remarks, INDEX_FILE_PATH};
use cargo_remark::utils::callback::ProgressBarCallback;
use cargo_remark::utils::timing::time_block_log_info;
use clap::Parser;
use env_logger::Env;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
    /// Build a crate with optimizations, generate optimization remarks and render a website
    /// with remark summary.
    Build(SharedArgs),
    /// Wrap an arbitrary cargo command, while configuring it to generate remarks.
    Wrap(SharedArgs),
}

#[derive(clap::Parser, Debug)]
#[clap(trailing_var_arg = true)]
struct SharedArgs {
    /// Open the generated website after the build finishes.
    #[arg(long)]
    open: bool,

    /// Load remarks from external code (i.e. crate dependencies).
    /// Note that this may produce a large amount of data!
    #[arg(long)]
    external: bool,

    /// Optimization remark kinds that should be ignored.
    #[arg(
        long = "filter",
        default_values = cargo_remark::DEFAULT_KIND_FILTER
    )]
    filter_kind: Vec<String>,

    /// Additional arguments that will be passed to Cargo.
    cargo_args: Vec<String>,
}

fn generate_remarks(subcmd: CargoSubcommand, args: SharedArgs) -> anyhow::Result<()> {
    let SharedArgs {
        open,
        external,
        filter_kind,
        cargo_args,
    } = args;
    if !check_remark_dir_support()? {
        return Err(anyhow::anyhow!(
            "Your version of rustc does not support `-Zremark-dir`. Please use a nightly version newer than 4. 7. 2023."
        ));
    }
    let output = run_cargo(subcmd, cargo_args)?;

    let rustc_source_root = match get_rustc_source_root() {
        Ok(root) => Some(root),
        Err(error) => {
            log::warn!("Cannot find rustc source root: {error:?}");
            None
        }
    };

    let remarks = time_block_log_info("Remark loading", || {
        load_remarks_from_dir(
            output.yaml_dir,
            RemarkLoadOptions {
                external,
                source_dir: output.source_dir.clone(),
                filter_kind,
                rustc_source_root,
            },
            Some(&ProgressBarCallback::default()),
        )
    })?;
    time_block_log_info("Rendering", || {
        render_remarks(
            remarks,
            &output.source_dir,
            &output.web_dir,
            Some(&ProgressBarCallback::default()),
        )
    })?;

    log::info!("Website built into {}.", cli_format_path(&output.web_dir));

    let index_path = output.web_dir.join(INDEX_FILE_PATH);

    if open {
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
            Subcommand::Build(args) => generate_remarks(CargoSubcommand::Build, args),
            Subcommand::Wrap(args) => generate_remarks(CargoSubcommand::Wrap, args),
        },
    }
}
