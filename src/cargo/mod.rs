use anyhow::Context;
use cargo_remark::RustcSourceRoot;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cargo_remark::utils::io::ensure_directory;

use crate::cargo::cli::cli_format_path;

pub mod cli;
pub mod version;

pub enum CargoSubcommand {
    Build,
    Wrap,
}

pub struct BuildOutput {
    pub out_dir: PathBuf,
    pub source_dir: PathBuf,
    pub gen_dir: PathBuf,
}

pub fn run_cargo(subcmd: CargoSubcommand, cargo_args: Vec<String>) -> anyhow::Result<BuildOutput> {
    let ctx = get_cargo_ctx()?;
    let remark_dir = ctx.get_target_directory(Path::new("remarks"))?;

    let gen_dir = ensure_directory(&remark_dir.join("gen"))?;

    log::info!(
        "Optimization remarks will be stored into {}.",
        cli_format_path(&gen_dir)
    );

    let mut cmd = match subcmd {
        CargoSubcommand::Build => {
            let cargo_args = parse_cargo_args(cargo_args);
            let mut cargo = Command::new("cargo");
            cargo
                .arg("build")
                .arg("--release")
                .stdin(Stdio::null())
                .args(cargo_args.filtered);
            cargo
        }
        CargoSubcommand::Wrap => {
            if cargo_args.is_empty() {
                return Err(anyhow::anyhow!("You have to enter a command after `--` that will be executed when using `wrap`."));
            };

            let mut cmd = Command::new("cargo");
            cmd.args(&cargo_args).stdin(Stdio::null());
            cmd
        }
    };

    let flags = format!(
        "-Cremark=all -Zremark-dir={} -Cdebuginfo=1",
        gen_dir.display()
    );
    set_cargo_env(&mut cmd, &flags);

    let status = cmd
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

    let out_dir = ensure_directory(&remark_dir.join("out"))?;
    Ok(BuildOutput {
        out_dir,
        source_dir: ctx.root_directory,
        gen_dir,
    })
}

pub fn get_rustc_source_root() -> anyhow::Result<RustcSourceRoot> {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .context("Cannot get sysroot from `rustc`")?;
    let sysroot = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    RustcSourceRoot::from_sysroot(sysroot)
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
