use std::borrow::Cow;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Context;
use askama::Template;
use rust_embed::RustEmbed;

use crate::remark::index::RemarkIndex;
use crate::remark::{Location, MessagePart};

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

pub fn render_remarks(index: RemarkIndex, output_dir: &Path) -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all(output_dir);
    std::fs::create_dir_all(output_dir).context("Cannot create output directory")?;

    // Create index page
    let remarks = index
        .remarks()
        .iter()
        .map(|r| RemarkEntry {
            name: &r.name,
            location: r
                .function
                .location
                .as_ref()
                .map(|l| render_location(l, None)),
            function: &r.function.name,
            message: format_message(&r.message),
        })
        .collect::<Vec<_>>();
    let index_page = IndexTemplate { remarks: &remarks };
    render_to_file(&index_page, &output_dir.join("index.html"))?;

    // Copy all static assets to the output directory
    for asset_path in StaticAssets::iter() {
        let data = StaticAssets::get(&asset_path).unwrap().data;
        let path = output_dir.join("assets").join(asset_path.as_ref());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Cannot create output asset directory")?;
        }
        std::fs::write(path, data).context("Cannot copy asset file to output directory")?;
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
    path.replace('/', "_")
}

// for (source_file, remarks) in index.files() {
//     let path = resolve_path(source_dir, &source_file);
//     let file = std::fs::read_to_string(&path)
//         .with_context(|| format!("Cannot read source file {}", path.display()))?;
//     println!("{}", file);
// }
// fn resolve_path<'a>(root_dir: &Path, path: &'a Path) -> Cow<'a, Path> {
//     if path.is_absolute() {
//         path.into()
//     } else {
//         root_dir.join(path).into()
//     }
// }

fn render_to_file<T: askama::Template>(template: &T, path: &Path) -> anyhow::Result<()> {
    let file = File::create(path)
        .with_context(|| format!("Cannot create template file {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    template
        .write_into(&mut writer)
        .with_context(|| format!("Cannot render template into {}", path.display()))?;
    Ok(())
}
