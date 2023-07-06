use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cargo_remark::remark::load_remarks_from_dir;
use cargo_remark::render::render_remarks;
use cargo_remark::utils::callback::ProgressBarCallback;
use cargo_remark::utils::io::ensure_directory;
use cargo_remark::utils::timing::time_block_log_info;

use crate::cargo::cli::cli_format_path;

mod cli;
pub mod version;

pub struct BuildOutput {
    pub out_dir: PathBuf,
}

pub fn build(cargo_args: Vec<String>) -> anyhow::Result<BuildOutput> {
    let ctx = get_cargo_ctx()?;
    let remark_dir = ctx.get_target_directory(Path::new("remarks"))?;

    let gen_dir = ensure_directory(&remark_dir.join("gen"))?;

    log::info!(
        "Optimization remarks will be stored into {}.",
        cli_format_path(&gen_dir)
    );

    let cargo_args = parse_cargo_args(cargo_args);
    let flags = format!(
        "-Cremark=all -Zremark-dir={} -Cdebuginfo=1",
        gen_dir.display()
    );
    let mut cargo = Command::new("cargo");
    cargo
        .arg("build")
        .arg("--release")
        .stdin(Stdio::null())
        .args(cargo_args.filtered);
    set_cargo_env(&mut cargo, &flags);

    let status = cargo
        .spawn()
        .map_err(|error| anyhow::anyhow!("Cannot start cargo: {error:?}"))?
        .wait()
        .map_err(|error| anyhow::anyhow!("Cargo failed: {error:?}"))?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "Cargo build failed: exit code {}",
            status.code().unwrap_or(1)
        ));
    }

    log::info!("Optimization remarks sucessfully generated");

    let source_dir = &ctx.root_directory;
    let out_dir = ensure_directory(&remark_dir.join("out"))?;
    let remarks = time_block_log_info("Remark loading", || {
        load_remarks_from_dir(gen_dir, Some(&ProgressBarCallback::default()))
    })?;
    time_block_log_info("Rendering", || {
        render_remarks(
            remarks,
            source_dir,
            &out_dir,
            Some(&ProgressBarCallback::default()),
        )
    })?;

    log::info!("Website built into {}.", cli_format_path(&out_dir));

    Ok(BuildOutput { out_dir })
}

fn set_cargo_env(command: &mut Command, flags: &str) {
    use std::fmt::Write;

    let mut rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
    write!(&mut rustflags, " {}", flags).unwrap();

    command.env("RUSTFLAGS", rustflags);
}

#[derive(Debug, Default)]
struct CargoArgs {
    filtered: Vec<String>,
}

fn parse_cargo_args(cargo_args: Vec<String>) -> CargoArgs {
    let mut args = CargoArgs::default();

    for arg in cargo_args {
        match arg.as_str() {
            // Skip `--release`, we will pass it by ourselves.
            "--release" => {
                log::warn!("Do not pass `--release` manually, it will be added automatically by `cargo-remark`");
            }
            _ => args.filtered.push(arg),
        }
    }
    args
}

struct CargoContext {
    target_directory: PathBuf,
    root_directory: PathBuf,
}

impl CargoContext {
    fn get_target_directory(&self, path: &Path) -> anyhow::Result<PathBuf> {
        let directory = self.target_directory.join(path);
        ensure_directory(&directory)?;
        Ok(directory)
    }
}

/// Finds Cargo metadata from the current directory.
fn get_cargo_ctx() -> anyhow::Result<CargoContext> {
    let cmd = cargo_metadata::MetadataCommand::new();
    let metadata = cmd
        .exec()
        .map_err(|error| anyhow::anyhow!("Cannot get cargo metadata: {:?}", error))?;
    Ok(CargoContext {
        target_directory: metadata.target_directory.into_std_path_buf(),
        root_directory: metadata.workspace_root.into_std_path_buf(),
    })
}
