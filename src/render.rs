use std::borrow::Cow;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Context;
use askama::Template;
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use rust_embed::RustEmbed;

use crate::remark::{Location, MessagePart, Remark};
use crate::utils::callback::LoadCallback;
use crate::utils::data_structures::Map;

#[derive(RustEmbed)]
#[folder = "templates/assets"]
struct StaticAssets;

#[derive(serde::Serialize)]
struct RemarkEntry<'a> {
    name: &'a str,
    location: Option<String>,
    function: &'a str,
    message: String,
}

#[derive(Template)]
#[template(path = "index.jinja")]
pub struct IndexTemplate<'a> {
    remarks: &'a [RemarkEntry<'a>],
}

#[derive(Template)]
#[template(path = "source-file.jinja")]
pub struct SourceFileTemplate<'a> {
    path: &'a str,
    remarks: Vec<&'a RemarkEntry<'a>>,
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

    let mut file_to_remarks: Map<&str, Vec<u32>> = Map::default();

    // Create index page
    let remark_entries = remarks
        .iter()
        .enumerate()
        .map(|(index, r)| {
            let entry = RemarkEntry {
                name: &r.name,
                location: r
                    .function
                    .location
                    .as_ref()
                    .map(|l| render_location(l, None)),
                function: &r.function.name,
                message: format_message(&r.message),
            };
            if let Some(ref location) = r.function.location {
                file_to_remarks
                    .entry(&location.file)
                    .or_default()
                    .push(index.try_into().expect("Too many remarks per file"));
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
    let index_page = IndexTemplate {
        remarks: &remark_entries,
    };
    render_to_file(&index_page, &output_dir.join("index.html"))?;

    if let Some(callback) = callback {
        callback.start(file_to_remarks.len() as u64);
    }

    // Render all found source files
    file_to_remarks
        .into_par_iter()
        .map(|(source_file, remark_indices)| -> anyhow::Result<()> {
            let original_path = resolve_path(source_dir, Path::new(source_file));
            let file_content = std::fs::read_to_string(&original_path)
                .with_context(|| format!("Cannot read source file {}", original_path.display()))?;

            if let Some(callback) = callback {
                callback.advance();
            }

            // TODO: deduplicate links to "self" (the same source file)
            let file_remarks: Vec<_> = remark_indices
                .into_iter()
                .map(|index| &remark_entries[index as usize])
                .collect();
            let output_path = output_dir.join(path_to_relative_url(source_file));
            let source_file_page = SourceFileTemplate {
                path: source_file,
                remarks: file_remarks,
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
            MessagePart::String(string) => buffer.push_str(string),
            MessagePart::AnnotatedString { message, location } => {
                buffer.push_str(&render_location(location, Some(message)))
            }
        }
    }
    buffer
}

// TODO: remove allocation(s) by writing into a fmt buffer
fn render_location(location: &Location, label: Option<&str>) -> String {
    let label = label.map(Cow::from).unwrap_or_else(|| {
        format!("{}:{}:{}", location.file, location.line, location.column).into()
    });

    format!(
        "<a href='{}#L{}'>{label}</a>",
        path_to_relative_url(&location.file),
        location.line
    )
}

fn path_to_relative_url(path: &str) -> String {
    format!("{}.html", path.replace('/', "_"))
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
