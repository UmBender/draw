//! Control: the same unit mixes fine, so the compile-fail cases fail for the
//! unit and not for a broken harness.
use draw_types::units::{Len, Point, World};

pub fn same_unit() -> Len<World> {
    let a = Point::<World>::new(0.0, 0.0);
    let b = Point::<World>::new(1.0, 1.0);
    a.distance(b) + Len::new(1.0)
}
