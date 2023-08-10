use crate::render::INDEX_FILE_PATH;
use crate::utils::cli::cli_format_path;
use std::path::Path;

pub mod callback;
pub mod cli;
pub mod data_structures;
pub mod io;
pub mod timing;

pub fn open_result(dir: &Path, open: bool) -> anyhow::Result<()> {
    let index_path = dir.join(INDEX_FILE_PATH);
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
