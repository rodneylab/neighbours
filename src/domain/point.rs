use crate::utilities::AppError;
use serde::Deserialize;
use std::{
    f64::consts::{FRAC_PI_2, PI},
    fs::read_to_string,
    path::Path,
};

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
    let PointList { points } = serde_json::from_str(&json).map_err(AppError::JSONParseError)?;
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

/// Angular position (or bearing) in radians, measured clockwise, between `0`
/// and `2 PI`, with `0` indicating point 2 is directly above the first.
fn angular_position((x_1, y_1): (i32, i32), (x_2, y_2): (i32, i32)) -> f64 {
    let horizontal_distance: f64 = (x_2 - x_1).into();
    let vertical_distance: f64 = (y_2 - y_1).into();

    if vertical_distance.abs() < 1e-10 {
        match horizontal_distance {
            val if val > 0.0 => return FRAC_PI_2,
            _ => return 3.0 * FRAC_PI_2,
        }
    }

    let theta = (horizontal_distance / vertical_distance).atan();

    if vertical_distance >= 0.0 {
        if horizontal_distance >= 0.0 {
            theta
        } else {
            (2.0 * PI) + theta
        }
    } else {
        PI + theta
    }
}

/// Returns true if `bearing` is inside segment sweeping counter-clockwise from
/// `center` by `half_arc_central_angle`.  `half_arc_central_angle` should be
/// between zero and `PI`.
fn inside_left_segment(bearing: f64, center: f64, half_arc_central_angle_radians: f64) -> bool {
    match center - half_arc_central_angle_radians {
        // left segment radius wraps through 0 radians
        val if val < 0.0 => {
            ((val + 2.0 * PI)..=(2.0 * PI)).contains(&bearing) || (0.0..=center).contains(&bearing)
        }
        val if val >= 0.0 => (val..=center).contains(&bearing),
        _ => unreachable!("Unexpected error checking bearing is inside left segment"),
    }
}

/// returns true if `bearing` is inside segment sweeping clockwise from `center`
/// by `half_arc_central_angle`.  `half_arc_central_angle` should be between
/// zero and `PI`.
fn inside_right_segment(bearing: f64, center: f64, half_arc_central_angle_radians: f64) -> bool {
    match center + half_arc_central_angle_radians {
        val if val < 2.0 * PI => (center..=val).contains(&bearing),

        // right segment radius wraps through `2 * PI` radians
        val if val >= 2.0 * PI => {
            (center..=(2.0 * PI)).contains(&bearing) || (0.0..=(val - 2.0 * PI)).contains(&bearing)
        }
        _ => unreachable!("Unexpected error checking bearing is inside right segment"),
    }
}

/// Helper function to determine if the second point is visible from the first,
/// taking into account the direction of the first point.  Returns true if the
/// second point is within a segment of large radius, sweeping left and right
/// from the first point’s direction by `half_arc_central_angle`.
/// `half_arc_central_angle` should be in degrees and lie in the range zero to
/// `180` degrees.
fn visible_neighbour(
    Point {
        coordinates: point_coordinates,
        direction,
        ..
    }: &Point,
    Point {
        coordinates: neighbour_coordinates,
        ..
    }: &Point,
    half_arc_central_angle: u32,
) -> bool {
    let bearing = angular_position(*point_coordinates, *neighbour_coordinates);
    let half_arc_central_angle_radians = (half_arc_central_angle as f64).to_radians();

    // direction point is facing
    let center: f64 = match direction {
        Direction::North => 0.0,
        Direction::East => FRAC_PI_2,
        Direction::South => PI,
        Direction::West => 3.0 * FRAC_PI_2,
    };

    // left segment sweeps left from center through an angle of
    // `half_arc_central_angle`
    // right segment sweeps right from center through an angle of
    // `half_arc_central_angle`
    inside_left_segment(bearing, center, half_arc_central_angle_radians)
        || inside_right_segment(bearing, center, half_arc_central_angle_radians)
}

/// Return a vector of all `neighbourhood` points within a segment whose centre
/// is at `point`, and has radius of `radius` units and spans left and right
/// front `point`’s direction by `half_arc_central_angle`.
/// `half_arc_central_angle` should be in degrees, and can range from zero to
/// `180` degrees.  `point` is never included in the returned vector.
fn close_neighbours<'a>(
    point: &'a Point,
    half_arc_central_angle: u32,
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
            if distance < radius as f64 && visible_neighbour(point, val, half_arc_central_angle) {
                acc.push(val);
            }
        }
        acc
    });
    result
}

/// Return a vector of all `neighbourhood` points within a segment whose centre
/// is at the starting point, identified by `point_number`, and has radius of
/// `radius` units, and spans left and right front `point`’s direction by
/// `half_arc_central_angle`.  `half_arc_central_angle` should be in degrees,
/// and can range from zero to `180` degrees.
///
/// An empty vector is returned if no point matching `point_number` is found
/// in neighbourhood. The starting point is never included in the returned
/// vector.  No checks are performed to ensure neighbourhood points have
/// unique numbers.
pub fn visible_points_from_neighbours(
    point_number: u32,
    half_arc_central_angle: u32,
    arc_radius: u32,
    neighbourhood: &[Point],
) -> Vec<&Point> {
    match neighbourhood
        .iter()
        .find(|Point { number, .. }| *number == point_number)
    {
        Some(value) => close_neighbours(value, half_arc_central_angle, arc_radius, neighbourhood),
        None => vec![],
    }
}

/// Return a vector of all neighbourhood points within a segment whose centre
/// is at the starting point, identified by `point_number`, and has radius of
/// `radius` units and spans left and right front `point`’s direction by
/// `half_arc_central_angle`.  `half_arc_central_angle` should be in degrees,
/// and can range from zero to `180` degrees.
///
/// An empty vector is returned if no point matching `point_number` is found
/// in neighbourhood. The starting point is never included in the returned
/// vector.  No checks are performed to ensure neighbourhood points have
/// unique numbers.  The universe of all points is read from `./points.json`.
pub fn visible_points(
    point_number: u32,
    arc_central_angle: u32,
    arc_radius: u32,
) -> Result<Vec<Point>, AppError> {
    let points_file_path = Path::new("./points.json");
    let points = parse_points_file(points_file_path)?;
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
        Direction, Point, angular_position, euclidean_distance, parse_points_file, visible_points,
        visible_points_from_neighbours,
    };
    use crate::utilities::AppError;
    use std::{
        f64::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2},
        path::Path,
    };

    #[test]
    fn angular_position_gives_expected_result() {
        // arrange
        let point_1 = (0, 0);
        let point_2 = (3, 3);

        // act
        let outcome = angular_position(point_1, point_2);

        // assert
        let abs_difference = (outcome - FRAC_PI_4).abs();
        assert!(abs_difference < 1e-10);

        // arrange
        let point_1 = (1, 1);
        let point_2 = (3, -1);

        // act
        let outcome = angular_position(point_1, point_2);

        // assert
        let abs_difference = (outcome - 3.0 * FRAC_PI_4).abs();
        assert!(abs_difference < 1e-10);

        // arrange
        let point_1 = (3, 1);
        let point_2 = (0, -2);

        // act
        let outcome = angular_position(point_1, point_2);

        // assert
        let abs_difference = (outcome - 5.0 * FRAC_PI_4).abs();
        assert!(abs_difference < 1e-10);

        // arrange
        let point_1 = (1, 1);
        let point_2 = (-1, 3);

        // act
        let outcome = angular_position(point_1, point_2);

        // assert
        let abs_difference = (outcome - 7.0 * FRAC_PI_4).abs();
        assert!(abs_difference < 1e-10);

        // arrange
        let point_1 = (2, 1);
        let point_2 = (2, 2);

        // act
        let outcome = angular_position(point_1, point_2);

        // assert
        let abs_difference = (outcome - 0.0).abs();
        assert!(abs_difference < 1e-10);

        // arrange
        let point_1 = (1, 0);
        let point_2 = (2, 0);

        // act
        let outcome = angular_position(point_1, point_2);

        // assert
        let abs_difference = (outcome - FRAC_PI_2).abs();
        assert!(abs_difference < 1e-10);

        // arrange
        let point_1 = (2, 0);
        let point_2 = (1, 0);

        // act
        let outcome = angular_position(point_1, point_2);

        // assert
        let abs_difference = (outcome - 3.0 * FRAC_PI_2).abs();
        assert!(abs_difference < 1e-10);
    }

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
        let outcome = euclidean_distance(point_1, point_2);
        let abs_difference = (outcome - SQRT_2).abs();

        // assert
        assert!(abs_difference < 1e-10);

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
        let points = parse_points_file(points_file_path)?;

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
        assert_eq!(
            outcome,
            "Error parsing JSON. Check the input JSON is valid and has expected structure: EOF while parsing a value at line 9 column 0"
        );
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
            outcome,
            "Error reading input file: `./fixtures/does-not-exist.json`. Check it exists and contains valid UTF-8."
        );
    }

    #[test]
    fn visible_points_handles_valid_input() -> Result<(), AppError> {
        // arrange

        // act
        let outcome = visible_points(1, 180, 20)?;

        // assert
        assert_eq!(outcome.len(), 10);

        // act
        let outcome = visible_points(1, 45, 20)?;

        // assert
        assert_eq!(outcome.len(), 1);
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
        let outcome = visible_points_from_neighbours(20, 180, 10, &points);

        // assert
        assert_eq!(outcome.len(), 2);
        assert!(
            outcome
                .iter()
                .find(|Point { number, .. }| *number == 5)
                .is_some()
        );
        assert!(
            outcome
                .iter()
                .find(|Point { number, .. }| *number == 6)
                .is_some()
        );

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
        let outcome = visible_points_from_neighbours(19, 60, 30, &points);

        // assert
        assert_eq!(outcome.len(), 1);

        // act
        let outcome = visible_points_from_neighbours(20, 70, 10, &points);

        // assert
        assert_eq!(outcome.len(), 0);

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
                direction: Direction::East,
            },
        ];

        // act
        let outcome = visible_points_from_neighbours(20, 70, 10, &points);

        // assert
        assert_eq!(outcome.len(), 2);
        assert!(
            outcome
                .iter()
                .find(|Point { number, .. }| *number == 5)
                .is_some()
        );
        assert!(
            outcome
                .iter()
                .find(|Point { number, .. }| *number == 6)
                .is_some()
        );
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
