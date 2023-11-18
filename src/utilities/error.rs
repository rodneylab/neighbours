use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(
        "Error reading input file: `{expected_path}`. Check it exists and contains valid UTF-8."
    )]
    InvalidFileError {
        expected_path: String,
        source: std::io::Error,
    },

    #[error("Error parsing JSON. Check the input JSON is valid and has expected structure: {0}")]
    JSONParseError(serde_json::Error),
}
