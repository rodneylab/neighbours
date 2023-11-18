mod domain;
mod utilities;

use crate::domain::visible_points;

/// Prints visible points taking point neighbourhood from `./points.json` input
/// file, which must exist.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let visible_points = match visible_points(1, 45, 20) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            return Err(error.into());
        }
    };

    println!("There are{} visible points", visible_points.len());
    println!("{:?}", visible_points);
    Ok(())
}
