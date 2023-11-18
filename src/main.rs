mod domain;
mod utilities;

use crate::domain::parse_points_file;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let points_file_path = Path::new("./points.json");
    if let Err(error) = parse_points_file(points_file_path) {
        eprintln!("{error}");
        return Err(error.into());
    }
    Ok(())
}
