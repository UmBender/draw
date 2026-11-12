// expect: mismatched types
use draw_types::units::{Len, Point, Screen, World};

pub fn mix() -> Len<World> {
    let w = Point::<World>::new(0.0, 0.0);
    let s = Point::<Screen>::new(1.0, 1.0);
    w.distance(s)
}
