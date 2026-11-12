// expect: mismatched types
use draw_types::units::{Point, Screen, World};

pub fn cast() -> Point<Screen> {
    // Only `Camera` converts; a world point is not a screen point.
    Point::<World>::new(1.0, 2.0)
}
