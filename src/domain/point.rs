use crate::utilities::AppError;
use serde::Deserialize;
use std::{fs::read_to_string, path::Path};

/// Represents direction faced by a point
#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

/// Represents a point as used internally
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    /// x,y coordinates of the point
    pub coordinates: (i32, i32),
    pub number: u32,
    pub direction: Direction,
}

/// Represents a point as found in an input file
#[derive(Debug, Deserialize)]
pub struct InputPoint {
    pub x: i32,
    pub y: i32,
    pub number: u32,
    pub direction: Direction,
}

/// List of points as found in a points JSON file
#[derive(Debug, Deserialize)]
pub struct PointList {
    pub points: Vec<InputPoint>,
}

/// Helper function for parsing a JSON file of points into a [`Vec`] of
/// [`Point`]s
pub fn parse_points_file<P: AsRef<Path>>(path: P) -> Result<Vec<Point>, AppError> {
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

/// Distance between two points
fn euclidean_distance((x_1, y_1): (i32, i32), (x_2, y_2): (i32, i32)) -> f64 {
    let horizontal_distance: f64 = (x_2 - x_1).into();
    let vertical_distance: f64 = (y_2 - y_1).into();
    ((horizontal_distance * horizontal_distance) + (vertical_distance * vertical_distance)).sqrt()
}

/// Return a vector of all points within `radius` units of `point`.  `point` is
/// never included in the returned vector.
fn close_neighbours<'a>(
    point: &'a Point,
    radius: u32,
    neighbourhood: &'a [Point],
) -> Vec<&'a Point> {
    let Point {
        number: point_number,
        coordinates: point_coordinates,
        ..
    } = point;
    let result: Vec<&Point> = neighbourhood.iter().fold(vec![], |mut acc, val| {
        let Point {
            number: neighbour_number,
            coordinates: neighbour_coordinates,
            ..
        } = val;
        if point_number != neighbour_number {
            let distance = euclidean_distance(*point_coordinates, *neighbour_coordinates);
            if distance < radius as f64 {
                acc.push(val);
            }
        }
        acc
    });
    result
}

/// Return a vector of all points within `arc_radius` units of a given starting
/// point.  The starting point is identified by `point_number`, and the
/// universe of all points is passed as the neighbourhood.  An empty vector is
/// returned if no point matching `point_number` is found in neighbourhood.
/// The starting point is never included in the returned vector.
pub fn visible_points_from_neighbours(
    point_number: u32,
    _arc_central_angle: u32,
    arc_radius: u32,
    neighbourhood: &[Point],
) -> Vec<&Point> {
    match neighbourhood
        .iter()
        .find(|Point { number, .. }| *number == point_number)
    {
        Some(value) => close_neighbours(value, arc_radius, neighbourhood),
        None => vec![],
    }
}

/// Return a vector of all points within arc_radius units of a given starting
/// point.  The starting point is identified by `point_number`, and the
/// universe of all points is read from `./points.json`.  An empty vector is
/// returned if no point matching `point_number` is found in neighbourhood.
/// The starting point is never included in the returned vector.
pub fn visible_points(
    point_number: u32,
    arc_central_angle: u32,
    arc_radius: u32,
) -> Result<Vec<Point>, AppError> {
    let points_file_path = Path::new("./points.json");
    let points = match parse_points_file(points_file_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            return Err(error);
        }
    };

    let result: Vec<Point> =
        visible_points_from_neighbours(point_number, arc_central_angle, arc_radius, &points)
            .iter()
            .map(|val| **val)
            .collect();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{
        euclidean_distance, parse_points_file, visible_points, visible_points_from_neighbours,
        Direction, Point,
    };
    use crate::utilities::AppError;
    use std::path::Path;

    #[test]
    fn euclidean_distance_gives_expected_result() {
        // arrange
        let point_1 = (1, 1);
        let point_2 = (4, 5);

        // act
        let distance = euclidean_distance(point_1, point_2);

        // assert
        assert_eq!(distance, 5.0);

        // arrange
        let point_1 = (-1, -1);
        let point_2 = (-2, -2);

        // act
        let distance = euclidean_distance(point_1, point_2);

        // assert
        assert_eq!(distance, 2.0_f64.sqrt());

        // arrange
        let point_1 = (0, 0);
        let point_2 = (0, 0);

        // act
        let distance = euclidean_distance(point_1, point_2);

        // assert
        assert_eq!(distance, 0.0);
    }

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

    #[test]
    fn visible_points_handles_valid_input() -> Result<(), AppError> {
        // arrange

        // act
        let outcome = visible_points(1, 45, 20)?;

        // assert
        assert_eq!(outcome.len(), 10);
        Ok(())
    }

    #[test]
    fn visible_points_from_neighbours_handles_valid_input() {
        // arrange
        let points: Vec<Point> = vec![
            Point {
                coordinates: (8, 6),
                number: 5,
                direction: Direction::North,
            },
            Point {
                coordinates: (6, 19),
                number: 6,
                direction: Direction::East,
            },
            Point {
                coordinates: (28, 26),
                number: 19,
                direction: Direction::South,
            },
            Point {
                coordinates: (2, 12),
                number: 20,
                direction: Direction::West,
            },
        ];

        // act
        let outcome = visible_points_from_neighbours(20, 45, 10, &points);

        // assert
        assert_eq!(outcome.len(), 2);
        assert!(outcome
            .iter()
            .find(|Point { number, .. }| *number == 5)
            .is_some());
        assert!(outcome
            .iter()
            .find(|Point { number, .. }| *number == 6)
            .is_some());
    }

    #[test]
    fn visible_points_from_neighbours_handles_empty_input_universe() {
        // arrange
        let points: Vec<Point> = vec![];

        // act
        let outcome = visible_points_from_neighbours(20, 45, 10, &points);

        // assert
        assert_eq!(outcome.len(), 0);
    }
}
