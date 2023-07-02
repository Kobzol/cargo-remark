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

    let remarks = time_block("Parse remark file", || parse_remarks(reader));
    Ok(remarks)
}

fn parse_remarks<R: std::io::Read>(reader: R) -> Vec<Remark> {
    let mut remarks = vec![];
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
                                    RemarkArg::Callee(inner) => RemarkArgument::Callee(Function {
                                        name: demangle(&inner.callee),
                                        location: inner.debug_loc.map(parse_debug_loc),
                                    }),
                                    RemarkArg::Caller(inner) => RemarkArgument::Caller(Function {
                                        name: demangle(&inner.caller),
                                        location: inner.debug_loc.map(parse_debug_loc),
                                    }),
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
    remarks
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

#[cfg(test)]
mod tests {
    use crate::remark::parse_remarks;

    #[test]
    fn parse_single() {
        let input = r#"--- !Missed
Pass:            sdagisel
Name:            FastISelFailure
Function:        __rust_alloc
DebugLoc:        { File: '/std/src/sys_common/backtrace.rs', 
                   Line: 131, Column: 0 }
Args:
  - String:          FastISel missed call
  - String:          ': '
  - String:          '  %3 = tail call ptr @__rdl_alloc(i64 %0, i64 %1)'
  - String:          ' (in function: __rust_alloc)'
..."#;
        insta::assert_debug_snapshot!(parse_remarks(input.as_bytes()), @r###"
        [
            Remark {
                pass: "sdagisel",
                name: "FastISelFailure",
                function: Function {
                    name: "__rust_alloc",
                    location: Some(
                        DebugLocation {
                            file: "/std/src/sys_common/backtrace.rs",
                            line: 131,
                            column: 0,
                        },
                    ),
                },
                args: [
                    String(
                        "FastISel missed call",
                    ),
                    String(
                        ": ",
                    ),
                    String(
                        "  %3 = tail call ptr @__rdl_alloc(i64 %0, i64 %1)",
                    ),
                    String(
                        " (in function: __rust_alloc)",
                    ),
                ],
            },
        ]
        "###);
    }

    #[test]
    fn parse_multiple() {
        let input = r#"--- !Missed
Pass:            inline
Name:            NoDefinition
DebugLoc:        { File: '/foo/rust/rust/library/std/src/rt.rs', 
                   Line: 165, Column: 17 }
Function:        _ZN3std2rt10lang_start17h9096f6f84fb08eb2E
Args:
  - Callee:          _ZN3std2rt19lang_start_internal17had90330d479f72f8E
  - String:          ' will not be inlined into '
  - Caller:          _ZN3std2rt10lang_start17h9096f6f84fb08eb2E
    DebugLoc:        { File: '/foo/rust/rust/library/std/src/rt.rs', 
                       Line: 159, Column: 0 }
  - String:          ' because its definition is unavailable'
...
--- !Missed
Pass:            inline
Name:            NoDefinition
DebugLoc:        { File: 'src/main.rs', Line: 7, Column: 5 }
Function:        _ZN7remarks4main17hc92ae132ef1efa8eE
Args:
  - Callee:          _ZN3std2io5stdio6_print17hdb04fec352560b87E
  - String:          ' will not be inlined into '
  - Caller:          _ZN7remarks4main17hc92ae132ef1efa8eE
    DebugLoc:        { File: 'src/main.rs', Line: 6, Column: 0 }
  - String:          ' because its definition is unavailable'
..."#;
        insta::assert_debug_snapshot!(parse_remarks(input.as_bytes()), @r###"
        [
            Remark {
                pass: "inline",
                name: "NoDefinition",
                function: Function {
                    name: "std::rt::lang_start::h9096f6f84fb08eb2",
                    location: Some(
                        DebugLocation {
                            file: "/foo/rust/rust/library/std/src/rt.rs",
                            line: 165,
                            column: 17,
                        },
                    ),
                },
                args: [
                    Callee(
                        Function {
                            name: "std::rt::lang_start_internal::had90330d479f72f8",
                            location: None,
                        },
                    ),
                    String(
                        " will not be inlined into ",
                    ),
                    Caller(
                        Function {
                            name: "std::rt::lang_start::h9096f6f84fb08eb2",
                            location: Some(
                                DebugLocation {
                                    file: "/foo/rust/rust/library/std/src/rt.rs",
                                    line: 159,
                                    column: 0,
                                },
                            ),
                        },
                    ),
                    String(
                        " because its definition is unavailable",
                    ),
                ],
            },
            Remark {
                pass: "inline",
                name: "NoDefinition",
                function: Function {
                    name: "remarks::main::hc92ae132ef1efa8e",
                    location: Some(
                        DebugLocation {
                            file: "src/main.rs",
                            line: 7,
                            column: 5,
                        },
                    ),
                },
                args: [
                    Callee(
                        Function {
                            name: "std::io::stdio::_print::hdb04fec352560b87",
                            location: None,
                        },
                    ),
                    String(
                        " will not be inlined into ",
                    ),
                    Caller(
                        Function {
                            name: "remarks::main::hc92ae132ef1efa8e",
                            location: Some(
                                DebugLocation {
                                    file: "src/main.rs",
                                    line: 6,
                                    column: 0,
                                },
                            ),
                        },
                    ),
                    String(
                        " because its definition is unavailable",
                    ),
                ],
            },
        ]
        "###);
    }

    #[test]
    fn parse_no_location() {
        let input = r#"--- !Missed
Pass:            sdagisel
Name:            FastISelFailure
Function:        __rust_alloc
Args:
  - String:          FastISel missed call
  - String:          ': '
  - String:          '  %3 = tail call ptr @__rdl_alloc(i64 %0, i64 %1)'
  - String:          ' (in function: __rust_alloc)'
..."#;
        insta::assert_debug_snapshot!(parse_remarks(input.as_bytes()), @r###"
        [
            Remark {
                pass: "sdagisel",
                name: "FastISelFailure",
                function: Function {
                    name: "__rust_alloc",
                    location: None,
                },
                args: [
                    String(
                        "FastISel missed call",
                    ),
                    String(
                        ": ",
                    ),
                    String(
                        "  %3 = tail call ptr @__rdl_alloc(i64 %0, i64 %1)",
                    ),
                    String(
                        " (in function: __rust_alloc)",
                    ),
                ],
            },
        ]
        "###);
    }

    #[test]
    fn parse_ignored_type() {
        let input = r#"--- !Passed
Pass:            inline
Name:            Inlined
DebugLoc:        { File: '/projects/personal/rust/rust/library/std/src/sys_common/backtrace.rs', 
                   Line: 135, Column: 18 }
Function:        _ZN3std10sys_common9backtrace28__rust_begin_short_backtrace17h7208ef7aa68440d8E
Args:
  - String:          ''''
  - Callee:          _ZN4core3ops8function6FnOnce9call_once17hde3380935eb1addfE
  - String:          ''' inlined into '''
  - Caller:          _ZN3std10sys_common9backtrace28__rust_begin_short_backtrace17h7208ef7aa68440d8E
    DebugLoc:        { File: '/projects/personal/rust/rust/library/std/src/sys_common/backtrace.rs', 
                       Line: 131, Column: 0 }
  - String:          ''''
  - String:          ' with '
  - String:          '(cost='
  - Cost:            '-15030'
  - String:          ', threshold='
  - Threshold:       '487'
  - String:          ')'
  - String:          ' at callsite '
  - String:          _ZN3std10sys_common9backtrace28__rust_begin_short_backtrace17h7208ef7aa68440d8E
  - String:          ':'
  - Line:            '4'
  - String:          ':'
  - Column:          '18'
  - String:          ';'
...
--- !Analysis
Pass:            size-info
Name:            FunctionMISizeChange
Function:        __rust_alloc
Args:
  - Pass:            Fast Register Allocator
  - String:          ': Function: '
  - Function:        __rust_alloc
  - String:          ': '
  - String:          'MI Instruction count changed from '
  - MIInstrsBefore:  '7'
  - String:          ' to '
  - MIInstrsAfter:   '1'
  - String:          '; Delta: '
  - Delta:           '-6'
..."#;
        assert!(parse_remarks(input.as_bytes()).is_empty());
    }
}
