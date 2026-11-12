// expect: private
use draw_types::ids::Id;

pub fn forge() -> Id<u8> {
    Id::<u8> {
        index: 0,
        generation: 0,
        marker: std::marker::PhantomData,
    }
}
