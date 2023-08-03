use std::borrow::Cow;
use std::fmt::Write;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;

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

#[derive(RustEmbed)]
#[folder = "templates/assets"]
struct StaticAssets;

#[derive(serde::Serialize)]
struct RemarkEntry<'a> {
    name: &'a str,
    location: Option<String>,
    function: Cow<'a, str>,
    message: Arc<str>,
}

#[derive(serde::Serialize, PartialEq, Eq, Hash)]
struct SourceRemark<'a> {
    name: &'a str,
    function: &'a str,
    line: Line,
    message: Arc<str>,
}

#[derive(Template)]
#[template(path = "index.jinja")]
pub struct IndexTemplate {
    remarks_json: String,
}

#[derive(Template)]
#[template(path = "source-file.jinja")]
pub struct SourceFileTemplate<'a> {
    path: &'a str,
    remarks: Set<SourceRemark<'a>>,
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

    let mut file_to_remarks: Map<&str, Set<SourceRemark>> = Map::default();

    // Create index page
    let remark_entries = remarks
        .iter()
        .map(|r| {
            let message: Arc<str> = format_message(&r.message).into();
            let entry = RemarkEntry {
                name: &r.name,
                location: r.function.location.as_ref().map(|l| {
                    let mut buffer = String::new();
                    render_location(&mut buffer, l, None);
                    buffer
                }),
                function: encode_safe(&r.function.name),
                message: message.clone(),
            };
            if let Some(ref location) = r.function.location {
                file_to_remarks
                    .entry(&location.file)
                    .or_default()
                    .insert(SourceRemark {
                        name: &r.name,
                        function: &r.function.name,
                        line: location.line,
                        message,
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
    let index_page = IndexTemplate {
        remarks_json: serialized_remarks,
    };
    render_to_file(&index_page, &output_dir.join(INDEX_FILE_PATH))?;

    if let Some(callback) = callback {
        callback.start(file_to_remarks.len() as u64);
    }

    // Render all found source files
    file_to_remarks
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
            path_to_relative_url(&mut buffer, source_file);
            let output_path = output_dir.join(buffer);
            let source_file_page = SourceFileTemplate {
                path: source_file,
                remarks,
                file_content,
            };
            render_to_file(&source_file_page, Path::new(&output_path))?;
            Ok(())
        })
        .collect::<Vec<anyhow::Result<()>>>();

    if let Some(callback) = callback {
        callback.finish();
    }

    Ok(())
}

fn format_message(parts: &[MessagePart]) -> String {
    let mut buffer = String::with_capacity(32);
    for part in parts {
        match part {
            MessagePart::String(string) => {
                encode_safe_to_string(string, &mut buffer);
            }
            MessagePart::AnnotatedString { message, location } => {
                render_location(&mut buffer, location, Some(message));
            }
        }
    }
    buffer
}

fn render_location(buffer: &mut String, location: &Location, label: Option<&str>) {
    buffer.push_str("<a href='");
    path_to_relative_url(buffer, &location.file);
    buffer.push_str("#L");
    buffer.write_fmt(format_args!("{}", location.line)).unwrap();
    buffer.push_str("'>");

    let label = label.map(Cow::from).unwrap_or_else(|| {
        format!("{}:{}:{}", location.file, location.line, location.column).into()
    });
    encode_safe_to_string(label, buffer);

    buffer.push_str("</a>");
}

fn path_to_relative_url(buffer: &mut String, path: &str) {
    write!(buffer, "{}.html", path.replace('/', "_")).unwrap()
}

fn resolve_path<'a>(root_dir: &Path, path: &'a Path) -> Cow<'a, Path> {
    if path.is_absolute() {
        path.into()
    } else {
        root_dir.join(path).into()
    }
}

fn render_to_file<T: askama::Template>(template: &T, path: &Path) -> anyhow::Result<()> {
    let file = File::create(path)
        .with_context(|| format!("Cannot create template file {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    template
        .write_into(&mut writer)
        .with_context(|| format!("Cannot render template into {}", path.display()))?;
    Ok(())
}
