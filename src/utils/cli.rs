use colored::{ColoredString, Colorize};
use std::path::Path;

/// Formats a path in a unified format to be printed in CLI.
pub fn cli_format_path<P: AsRef<Path>>(path: P) -> ColoredString {
    path.as_ref().display().to_string().yellow()
}
