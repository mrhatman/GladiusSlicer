use geo::Coord;
use crate::error::SlicerErrors;
use crate::messages::Message;
use crate::warning::SlicerWarnings;
use log::{error, info, warn};
use nalgebra::Vector2;
use std::{
    io::Write,
    time::{Duration, SystemTime},
};

///Current state information
pub struct StateContext {
    ///display type to use
    pub display_type: DisplayType,
    start_time: SystemTime,
    last_time: SystemTime,
}

impl StateContext {
    ///Create new State Context with given display type
    pub fn new(display_type: DisplayType) -> StateContext {
        let time = SystemTime::now();
        StateContext {
            display_type,
            start_time: time,
            last_time: time,
        }
    }

    ///Get elapsed time since last call to get elapsed time or new
    pub fn get_elapsed_time(&mut self) -> Duration {
        let time = SystemTime::now();
        let elapsed = SystemTime::now()
            .duration_since(self.last_time)
            .expect("Time can only go forward");
        self.last_time = time;
        elapsed
    }

    ///Get elapsed time since new
    pub fn get_total_elapsed_time(&self) -> Duration {
        let elapsed = SystemTime::now()
            .duration_since(self.start_time)
            .expect("Time can only go forward");
        elapsed
    }
}

///Types to display output messages
pub enum DisplayType {
    /// Output to log
    Message,
    /// Output to stdout
    StdOut,
}

///Logs at error level the given error
pub fn show_error_message(error: &SlicerErrors) {
    let (error_code, message) = error.get_code_and_message();
    error!("\n");
    error!("**************************************************");
    error!("\tGladius Slicer Ran into an error");
    error!("\tError Code: {:#X}", error_code);
    error!("\t{}", message);
    error!("**************************************************");
    error!("\n\n\n");
}

///Outputs the binary serial version of the error to stdout
pub fn send_error_message(error: SlicerErrors) {
    let stdout = std::io::stdout();
    let mut stdio_lock = stdout.lock();

    let message = Message::Error(error);
    bincode::serialize_into(&mut stdio_lock, &message).expect("Write Limit should not be hit");
    stdio_lock.flush().expect("Standard Out should be limited");
}

///Logs at warn level the given warning
pub fn show_warning_message(warning: &SlicerWarnings) {
    let (error_code, message) = warning.get_code_and_message();
    warn!("\n");
    warn!("**************************************************");
    warn!("\tGladius Slicer found a warning");
    warn!("\tWarning Code: {:#X}", error_code);
    warn!("\t{}", message);
    warn!("**************************************************");
    warn!("\n\n\n");
}

///Outputs the binary serial version of the warning to stdout
pub fn send_warning_message(warning: SlicerWarnings) {
    let stdout = std::io::stdout();
    let mut stdio_lock = stdout.lock();
    let message = Message::Warning(warning);
    bincode::serialize_into(&mut stdio_lock, &message).expect("Write Limit should not be hit");
    stdio_lock.flush().expect("Standard Out should be limited");
}


///Updates the current state and logs it given outout
pub fn state_update(state_message: &str, state_context: &mut StateContext) {
    match state_context.display_type {
        DisplayType::Message => {
            let stdout = std::io::stdout();
            let mut stdio_lock = stdout.lock();
            let message = Message::StateUpdate(state_message.to_string());
            bincode::serialize_into(&mut stdio_lock, &message)
                .expect("Write Limit should not be hit");
            stdio_lock.flush().expect("Standard Out should be limited");
        }
        DisplayType::StdOut => {
            let duration = state_context.get_elapsed_time();
            info!("{}\t{}", state_message, duration.as_millis());
        }
    }
}


///Calculate the point between a and b with the given y value
/// y must between a.y and b.y
#[inline]
pub fn point_y_lerp(a: &Coord<f64>, b: &Coord<f64>, y: f64) -> Coord<f64> {
    Coord {
        x: lerp(a.x, b.x, (y - a.y) / (b.y - a.y)),
        y,
    }
}

///Linear interoplate between points a and b at distance f(0.0-1.0)
#[inline]
pub fn point_lerp(a: &Coord<f64>, b: &Coord<f64>, f: f64) -> Coord<f64> {
    Coord {
        x: lerp(a.x, b.x, f),
        y: lerp(a.y, b.y, f),
    }
}

///Linear interoplate between a and b at distance f(0.0-1.0)
#[inline]
pub fn lerp(a: f64, b: f64, f: f64) -> f64 {
    a + f * (b - a)
}

/// Function to generate a unit bisector of the angle p0, p1, p2 that will always be inside the angle to the left
pub fn directional_unit_bisector_left(
    p0: &Coord<f64>,
    p1: &Coord<f64>,
    p2: &Coord<f64>,
) -> Vector2<f64> {
    let v1 = Vector2::new(p0.x - p1.x, p0.y - p1.y);
    let v2 = Vector2::new(p2.x - p1.x, p2.y - p1.y);

    let v1_scale = v1 * v2.magnitude();
    let v2_scale = v2 * v1.magnitude();

    let direction = v1_scale + v2_scale;

    match orientation(p0, p1, p2) {
        Orientation::Linear => {
            let perp = Vector2::new(-v1.y, v1.x).normalize();
            match orientation(p0, p1, &Coord::from((p1.x + perp.x, p1.y + perp.y))) {
                Orientation::Linear => {
                    unreachable!()
                }
                Orientation::Left => perp.normalize(),
                Orientation::Right => perp.normalize().scale(-1.0),
            }
        }
        Orientation::Left => direction.normalize(),
        Orientation::Right => direction.normalize().scale(-1.0),
    }
}

///Whether a set of three points curves to the left, right, or is linear
#[derive(Debug, PartialEq)]
pub enum Orientation {
    /// The 3 points are linear
    Linear,
    /// The points go to the left/are CCW
    Left,
    /// The points go to the right/are CW
    Right,
}

///Given a set of 3 points calculate its orientation
pub fn orientation(p: &Coord<f64>, q: &Coord<f64>, r: &Coord<f64>) -> Orientation {
    let left_val = (q.x - p.x) * (r.y - p.y);
    let right_val = (q.y - p.y) * (r.x - p.x);

    if left_val == right_val {
        Orientation::Linear
    } else if left_val > right_val {
        Orientation::Left
    } else {
        Orientation::Right
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_directional_unit_bisector() {
        assert_eq!(
            directional_unit_bisector_left(
                &Coord::from((0.0, 0.0)),
                &Coord::from((1.0, 0.0)),
                &Coord::from((1.0, 1.0))
            ),
            Vector2::new(-1.0, 1.0).normalize()
        );
        assert_eq!(
            directional_unit_bisector_left(
                &Coord::from((1.0, 1.0)),
                &Coord::from((1.0, 0.0)),
                &Coord::from((0.0, 0.0))
            ),
            Vector2::new(1.0, -1.0).normalize()
        );

        assert_eq!(
            directional_unit_bisector_left(
                &Coord::from((0.0, 0.0)),
                &Coord::from((1.0, 0.0)),
                &Coord::from((2.0, 0.0))
            ),
            Vector2::new(0.0, 1.0)
        );
        assert_eq!(
            directional_unit_bisector_left(
                &Coord::from((2.0, 0.0)),
                &Coord::from((1.0, 0.0)),
                &Coord::from((0.0, 0.0))
            ),
            Vector2::new(0.0, -1.0)
        );

        assert_eq!(
            directional_unit_bisector_left(
                &Coord::from((0.0, 0.0)),
                &Coord::from((0.0, 1.0)),
                &Coord::from((0.0, 1.0))
            ),
            Vector2::new(-1.0, 0.0)
        );
        assert_eq!(
            directional_unit_bisector_left(
                &Coord::from((0.0, 2.0)),
                &Coord::from((0.0, 1.0)),
                &Coord::from((0.0, 0.0))
            ),
            Vector2::new(1.0, 0.0)
        );
    }
}
