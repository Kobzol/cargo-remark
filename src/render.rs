use std::borrow::Cow;
use std::fmt::Write;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, MAIN_SEPARATOR};

use anyhow::Context;
use askama::Template;
use html_escape::{encode_safe, encode_safe_to_string};
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use rust_embed::RustEmbed;

use crate::remark::{Line, Location, MessagePart, Remark};
use crate::utils::callback::LoadCallback;
use crate::utils::data_structures::{Map, Set};

pub const INDEX_FILE_PATH: &str = "index.html";
const REMARK_LIST_FILE_PATH: &str = "remarks.html";

/// Directory where sources will be stored.
/// Relative to the output directory.
const SRC_DIR_NAME: &str = "src";

#[derive(RustEmbed)]
#[folder = "templates/assets"]
struct StaticAssets;

#[derive(serde::Serialize)]
struct RemarkIndexEntry<'a> {
    name: &'a str,
    location: Option<String>,
    function: Cow<'a, str>,
    message: String,
    hotness: Option<i32>,
}

#[derive(serde::Serialize, PartialEq, Eq, Hash)]
struct RemarkSourceEntry<'a> {
    name: &'a str,
    function: &'a str,
    line: Line,
    message: String,
    hotness: Option<i32>,
}

#[derive(Template)]
#[template(path = "remark-list.jinja")]
pub struct RemarkListTemplate {
    remarks_json: String,
}

#[derive(serde::Serialize)]
struct SourceFileLink<'a> {
    name: &'a str,
    file: String,
    remark_count: u64,
}

#[derive(Template)]
#[template(path = "index.jinja")]
pub struct IndexTemplate<'a> {
    source_links: Vec<SourceFileLink<'a>>,
}

#[derive(Template)]
#[template(path = "source-file.jinja")]
pub struct SourceFileTemplate<'a> {
    path: &'a str,
    remarks: Set<RemarkSourceEntry<'a>>,
    file_content: String,
}

pub fn render_remarks(
    remarks: Vec<Remark>,
    source_dir: &Path,
    output_dir: &Path,
    callback: Option<&(dyn LoadCallback + Sync)>,
) -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all(output_dir);
    std::fs::create_dir_all(output_dir).context("Cannot create output directory")?;

    // Copy all static assets to the output directory
    for asset_path in StaticAssets::iter() {
        let data = StaticAssets::get(&asset_path).unwrap().data;
        let path = output_dir.join("assets").join(asset_path.as_ref());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Cannot create output asset directory")?;
        }
        std::fs::write(path, data).context("Cannot copy asset file to output directory")?;
    }

    let mut file_to_remarks: Map<&str, Set<RemarkSourceEntry>> = Map::default();

    // Create remark list page
    let remark_entries = remarks
        .iter()
        .map(|r| {
            let Remark {
                pass: _,
                name,
                function,
                message,
                hotness,
            } = r;

            let entry = RemarkIndexEntry {
                name,
                location: function.location.as_ref().map(|location| {
                    let mut buffer = String::new();
                    render_remark_link(&mut buffer, location, Some(SRC_DIR_NAME), None);
                    buffer
                }),
                function: encode_safe(&function.name),
                message: format_message(message, Some(SRC_DIR_NAME)),
                hotness: *hotness,
            };
            if let Some(ref location) = function.location {
                file_to_remarks
                    .entry(&location.file)
                    .or_default()
                    .insert(RemarkSourceEntry {
                        name,
                        function: &function.name,
                        line: location.line,
                        // Inside the file, the link should be relative to the src directory
                        message: format_message(message, None),
                        hotness: *hotness,
                    });
            }
            // We also need to create file mappings for all referenced files, not just for files
            // with a remark.
            for msg_part in &r.message {
                if let MessagePart::AnnotatedString { location, .. } = msg_part {
                    file_to_remarks.entry(&location.file).or_default();
                }
            }
            entry
        })
        .collect::<Vec<_>>();

    let serialized_remarks = serde_json::to_string(&remark_entries)?;
    let remark_list_page = RemarkListTemplate {
        remarks_json: serialized_remarks,
    };
    render_to_file(&remark_list_page, &output_dir.join(REMARK_LIST_FILE_PATH))?;

    let mut source_links: Vec<SourceFileLink> = file_to_remarks
        .iter()
        .filter(|(_, remarks)| !remarks.is_empty())
        .map(|(name, remarks)| {
            let mut file = String::new();
            path_to_relative_url(&mut file, Some(SRC_DIR_NAME), name);
            SourceFileLink {
                name,
                file,
                remark_count: remarks.len() as u64,
            }
        })
        .collect();

    // Sort by relative files first, then in descending order by remark count
    source_links.sort_by_key(|link| (link.name.starts_with('/'), -(link.remark_count as i64)));

    let index_page = IndexTemplate { source_links };
    render_to_file(&index_page, &output_dir.join(INDEX_FILE_PATH))?;

    if let Some(callback) = callback {
        callback.start(file_to_remarks.len() as u64);
    }

    // Render all found source files
    let results = file_to_remarks
        .into_par_iter()
        .map(|(source_file, remarks)| -> anyhow::Result<()> {
            let original_path = resolve_path(source_dir, Path::new(source_file));
            let file_content = std::fs::read_to_string(&original_path)
                .with_context(|| format!("Cannot read source file {}", original_path.display()))?;

            if let Some(callback) = callback {
                callback.advance();
            }

            // TODO: deduplicate links to "self" (the same source file)
            let mut buffer = String::new();
            path_to_relative_url(&mut buffer, Some(SRC_DIR_NAME), source_file);
            let output_path = output_dir.join(buffer);
            let source_file_page = SourceFileTemplate {
                path: source_file,
                remarks,
                file_content,
            };
            render_to_file(&source_file_page, Path::new(&output_path))
                .with_context(|| anyhow::anyhow!("Failed to render {source_file}"))?;
            Ok(())
        })
        .collect::<Vec<anyhow::Result<()>>>();

    let failed = results.into_iter().filter(|r| r.is_err()).count();
    if failed > 0 {
        log::warn!("Failed to write {failed} source file(s)");
    }

    if let Some(callback) = callback {
        callback.finish();
    }

    Ok(())
}

fn format_message(parts: &[MessagePart], prefix: Option<&str>) -> String {
    let mut buffer = String::with_capacity(32);
    for part in parts {
        match part {
            MessagePart::String(string) => {
                encode_safe_to_string(string, &mut buffer);
            }
            MessagePart::AnnotatedString { message, location } => {
                render_remark_link(&mut buffer, location, prefix, Some(message));
            }
        }
    }
    buffer
}

fn render_remark_link(
    buffer: &mut String,
    location: &Location,
    prefix: Option<&str>,
    label: Option<&str>,
) {
    buffer.push_str("<a href='");
    path_to_relative_url(buffer, prefix, &location.file);
    buffer.push_str("#L");
    buffer.write_fmt(format_args!("{}", location.line)).unwrap();
    buffer.push_str("'>");

    let label = label.map(Cow::from).unwrap_or_else(|| {
        format!("{}:{}:{}", location.file, location.line, location.column).into()
    });
    encode_safe_to_string(label, buffer);

    buffer.push_str("</a>");
}

fn path_to_relative_url(buffer: &mut String, prefix: Option<&str>, path: &str) {
    if let Some(prefix) = prefix {
        buffer.push_str(prefix);
        buffer.push(MAIN_SEPARATOR);
    }
    for ch in path.chars() {
        if ch == '/' || ch == '\\' {
            buffer.push('_');
        } else {
            buffer.push(ch);
        }
    }
    buffer.push_str(".html");
}

fn resolve_path<'a>(root_dir: &Path, path: &'a Path) -> Cow<'a, Path> {
    if path.is_absolute() {
        path.into()
    } else {
        root_dir.join(path).into()
    }
}

fn render_to_file<T: askama::Template>(template: &T, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .context("Cannot create directory for storing rendered file")?;
    }

    let file = File::create(path)
        .with_context(|| format!("Cannot create template file {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    template
        .write_into(&mut writer)
        .with_context(|| format!("Cannot render template into {}", path.display()))?;
    Ok(())
}
