use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

use crate::remark::parse::RemarkArg;
use crate::utils::data_structures::Map;
use crate::utils::timing::time_block;

pub mod index;
mod parse;

/// We expect that the remark YAML files will have this extension.
const EXPECTED_EXTENSION: &str = ".opt.yaml";

#[derive(Debug)]
pub struct DebugLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub location: Option<DebugLocation>,
}

#[derive(Debug)]
pub enum RemarkArgument {
    String(String),
    Callee(Function),
    Caller(Function),
    Reason(String),
    Other(Map<String, String>),
}

#[derive(Debug)]
pub struct Remark {
    pub pass: String,
    pub name: String,
    pub function: Function,
    pub args: Vec<RemarkArgument>,
}

pub fn load_remarks_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Remark>> {
    let path = path.as_ref();

    let file =
        File::open(path).with_context(|| format!("Cannot open remark file {}", path.display()))?;
    log::debug!("Parsing {}", path.display());

    if file.metadata()?.len() == 0 {
        log::debug!("File is empty");
        return Ok(vec![]);
    }

    let reader = BufReader::new(file);
    let mut remarks = vec![];

    time_block("Parse remark file", || {
        for document in serde_yaml::Deserializer::from_reader(reader) {
            match parse::Remark::deserialize(document) {
                Ok(remark) => {
                    // TODO: optimize (intern)
                    match remark {
                        parse::Remark::Missed(remark) => {
                            let remark = Remark {
                                pass: remark.pass.to_string(),
                                name: remark.name.to_string(),
                                function: Function {
                                    name: demangle(&remark.function),
                                    location: remark.debug_loc.map(parse_debug_loc),
                                },
                                args: remark
                                    .args
                                    .into_iter()
                                    .map(|arg| match arg {
                                        RemarkArg::String(inner) => {
                                            RemarkArgument::String(inner.string.into_owned())
                                        }
                                        RemarkArg::Callee(inner) => {
                                            RemarkArgument::Callee(Function {
                                                name: demangle(&inner.callee),
                                                location: inner.debug_loc.map(parse_debug_loc),
                                            })
                                        }
                                        RemarkArg::Caller(inner) => {
                                            RemarkArgument::Caller(Function {
                                                name: demangle(&inner.caller),
                                                location: inner.debug_loc.map(parse_debug_loc),
                                            })
                                        }
                                        RemarkArg::Reason(inner) => {
                                            RemarkArgument::Reason(inner.reason.into_owned())
                                        }
                                        RemarkArg::Other(inner) => RemarkArgument::Other(inner),
                                    })
                                    .collect(),
                            };
                            remarks.push(remark);
                        }
                        parse::Remark::Passed {} => {}
                        parse::Remark::Analysis {} => {}
                    }
                }
                Err(error) => {
                    log::debug!("Error while deserializing remark: {error:?}");
                }
            }
        }
    });
    Ok(remarks)
}

pub fn load_remarks_from_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Remark>> {
    let dir = path.as_ref().to_path_buf().canonicalize()?;
    let files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .with_context(|| format!("Could not read remark directory {}", dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.file_type().ok()?.is_file() {
                return None;
            }
            if !entry
                .file_name()
                .to_str()
                .map(|extension| extension.ends_with(EXPECTED_EXTENSION))
                .unwrap_or(false)
            {
                return None;
            }
            Some(entry.path())
        })
        .collect();

    log::debug!("Parsing {} file(s) from {}", files.len(), dir.display());

    let remarks: Vec<(PathBuf, anyhow::Result<Vec<Remark>>)> = files
        .into_iter()
        .map(|file| {
            let remarks = load_remarks_from_file(&file);
            (file, remarks)
        })
        .collect();

    Ok(remarks
        .into_iter()
        .filter_map(|(path, result)| match result {
            Ok(remarks) => Some(remarks),
            Err(error) => {
                log::error!("Failed to load remarks from: {}: {error:?}", path.display());
                None
            }
        })
        .flatten()
        .collect())
}

fn parse_debug_loc(location: parse::DebugLocation) -> DebugLocation {
    DebugLocation {
        file: location.file.into_owned(),
        line: location.line,
        column: location.column,
    }
}

fn demangle(function: &str) -> String {
    rustc_demangle::demangle(function).to_string()
}
