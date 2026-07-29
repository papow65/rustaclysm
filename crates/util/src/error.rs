use anyhow::Error;
use std::path::Path;

/// Create a consistent JSON parsing error with file path and file contents
///
/// # Arguments
/// * `err` - The `serde_json::Error` that occurred
/// * `file_path` - Path to the JSON file
/// * `contents` - File contents (will be automatically truncated to 1000 characters)
///
/// # Returns
/// `anyhow::Error` with consistent formatting
pub fn json_parse_error(
    err: serde_json::Error,
    file_path: impl AsRef<Path>,
    contents: impl Into<String>,
) -> Error {
    let mut contents = contents.into();
    if let Some(upto) = contents.char_indices().map(|(i, _)| i).nth(1000) {
        contents.truncate(upto);
        contents += "...";
    }

    Error::new(err).context(format!(
        "Failed to parse JSON file: {}\nFile contents: {}",
        file_path.as_ref().display(),
        contents
    ))
}
