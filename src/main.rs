use serde::Deserialize;
use std::{fs::read_to_string, path::Path};
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

/// Represents direction faced by a point
#[derive(Debug, Deserialize, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

/// Represents a point as used internally
#[derive(Debug, PartialEq)]
struct Point {
    /// x,y coordinates of the point
    coordinates: (i32, i32),
    number: u32,
    direction: Direction,
}

/// Represents a point as found in an input file
#[derive(Debug, Deserialize)]
struct InputPoint {
    x: i32,
    y: i32,
    number: u32,
    direction: Direction,
}

/// List of points as found in a points JSON file
#[derive(Debug, Deserialize)]
struct PointList {
    points: Vec<InputPoint>,
}

/// Helper function for parsing a JSON file of points into a [`Vec`] of [`Point`]s
fn parse_points_file<P: AsRef<Path>>(path: P) -> Result<Vec<Point>, AppError> {
    let path_ref = path.as_ref();
    let json = match read_to_string(path_ref) {
        Ok(value) => value,
        Err(error) => {
            let expected_path = path_ref.display().to_string();
            return Err(AppError::InvalidFileError {
                expected_path,
                source: error,
            });
        }
    };
    let PointList { points } = match serde_json::from_str(&json).map_err(AppError::JSONParseError) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let result: Vec<Point> = points
        .into_iter()
        .map(
            |InputPoint {
                 x,
                 y,
                 number,
                 direction,
             }| Point {
                coordinates: (x, y),
                number,
                direction,
            },
        )
        .collect();
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let points_file_path = Path::new("./points.json");
    if let Err(error) = parse_points_file(points_file_path) {
        eprintln!("{error}");
        return Err(error.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_points_file, AppError, Direction, Point};
    use std::path::Path;

    #[test]
    fn parses_valid_points_file() -> Result<(), AppError> {
        // arrange
        let points_file_path = Path::new("./fixtures/valid_points.json");

        // act
        let points = parse_points_file(&points_file_path)?;

        // assert
        assert_eq!(points.len(), 20);
        assert_eq!(
            points[9],
            Point {
                coordinates: (36, 20),
                number: 10,
                direction: Direction::East
            }
        );
        Ok(())
    }

    #[test]
    fn handles_invalid_points_file() {
        // arrange
        let points_file_path = Path::new("./fixtures/invalid.json");

        // act
        let outcome = parse_points_file(&points_file_path)
            .unwrap_err()
            .to_string();

        // assert
        assert_eq!(outcome, "Error parsing JSON. Check the input JSON is valid and has expected structure: EOF while parsing a value at line 9 column 0");
    }

    #[test]
    fn handles_missing_points_file() {
        // arrange
        let points_file_path = Path::new("./fixtures/does-not-exist.json");

        // act
        let outcome = parse_points_file(&points_file_path)
            .unwrap_err()
            .to_string();

        // assert
        assert_eq!(
            outcome,"Error reading input file: `./fixtures/does-not-exist.json`. Check it exists and contains valid UTF-8."
        );
    }
}
