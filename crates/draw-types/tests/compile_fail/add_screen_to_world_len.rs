// expect: mismatched types
use draw_types::units::{Len, Screen, World};

pub fn add() -> Len<World> {
    Len::<World>::new(1.0) + Len::<Screen>::new(1.0)
}
