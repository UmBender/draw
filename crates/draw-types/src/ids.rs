//! Ids that cannot dangle: a generational [`Arena`] hands out [`Id`]s that have
//! no public constructor, and [`PageIx`] is always below [`PAGE_COUNT`].

use std::marker::PhantomData;

/// Number of pages in the editor.
pub const PAGE_COUNT: usize = 6;

/// A page number, always `< PAGE_COUNT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PageIx(u8);

impl PageIx {
    /// The first page.
    pub const FIRST: Self = Self(0);

    /// The page at `index`, or `None` if `index >= PAGE_COUNT`.
    #[must_use]
    pub fn new(index: usize) -> Option<Self> {
        if index < PAGE_COUNT {
            u8::try_from(index).ok().map(Self)
        } else {
            None
        }
    }

    /// The index, `< PAGE_COUNT`.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Handle to a value of an [`Arena<T>`]. Only the arena creates them, and an
/// id of a removed value is rejected even if its slot is reused.
pub struct Id<T> {
    index: u32,
    generation: u32,
    marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    /// The slot number (reused after a removal).
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Id<T> {}
impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}
impl<T> Eq for Id<T> {}
impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}
impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({}v{})", self.index, self.generation)
    }
}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// Generational arena: stable ids, slot reuse, stale ids rejected.
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    len: usize,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arena<T> {
    /// An empty arena.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            len: 0,
        }
    }

    /// Number of live values.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether no value is live.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Stores `value` and returns its id.
    ///
    /// # Panics
    /// Never in practice: more than `u32::MAX` slots would exhaust memory
    /// first; the id then saturates instead of panicking.
    pub fn insert(&mut self, value: T) -> Id<T> {
        self.len += 1;
        if let Some((index, slot)) = self
            .free
            .pop()
            .and_then(|i| Some((i, self.slots.get_mut(i as usize)?)))
        {
            slot.value = Some(value);
            return Id {
                index,
                generation: slot.generation,
                marker: PhantomData,
            };
        }
        let index = u32::try_from(self.slots.len()).unwrap_or(u32::MAX);
        self.slots.push(Slot {
            generation: 0,
            value: Some(value),
        });
        Id {
            index,
            generation: 0,
            marker: PhantomData,
        }
    }

    /// Removes and returns the value, or `None` for a stale id.
    pub fn remove(&mut self, id: Id<T>) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index);
        self.len -= 1;
        Some(value)
    }

    /// The value of `id`, or `None` for a stale id.
    #[must_use]
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_ref()
        } else {
            None
        }
    }

    /// The value of `id` for mutation, or `None` for a stale id.
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_mut()
        } else {
            None
        }
    }

    /// Whether `id` is live.
    #[must_use]
    pub fn contains(&self, id: Id<T>) -> bool {
        self.get(id).is_some()
    }

    /// Live values with their ids, in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (Id<T>, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, slot)| {
            let value = slot.value.as_ref()?;
            let index = u32::try_from(i).ok()?;
            Some((
                Id {
                    index,
                    generation: slot.generation,
                    marker: PhantomData,
                },
                value,
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn arena_insert_get_remove_round_trip() {
        let mut a = Arena::new();

        let id = a.insert("x");

        assert_eq!(a.get(id), Some(&"x"));
        assert_eq!(a.len(), 1);
        assert_eq!(a.remove(id), Some("x"));
        assert!(a.is_empty());
    }

    #[test]
    fn arena_insert_after_remove_reuses_slot_with_new_generation() {
        let mut a = Arena::new();
        let old = a.insert(1);
        a.remove(old);

        let new = a.insert(2);

        assert_eq!(old.index(), new.index());
        assert_ne!(old, new);
    }

    #[test]
    fn arena_stale_id_is_rejected() {
        let mut a = Arena::new();
        let old = a.insert(1);
        a.remove(old);
        let new = a.insert(2);

        assert_eq!(a.get(old), None);
        assert_eq!(a.get_mut(old), None);
        assert_eq!(a.remove(old), None);
        assert!(!a.contains(old));
        assert_eq!(a.get(new), Some(&2));
    }

    #[test]
    fn arena_iter_skips_removed() {
        let mut a = Arena::new();
        let x = a.insert(1);
        let y = a.insert(2);
        a.remove(x);

        let items: Vec<_> = a.iter().map(|(id, v)| (id, *v)).collect();

        assert_eq!(items, vec![(y, 2)]);
    }

    #[test]
    fn page_ix_rejects_out_of_range() {
        assert!(PageIx::new(PAGE_COUNT).is_none());
        assert!(PageIx::new(usize::MAX).is_none());
    }

    #[test]
    fn page_ix_round_trips_index() {
        for i in 0..PAGE_COUNT {
            assert_eq!(PageIx::new(i).map(PageIx::index), Some(i));
        }
        assert_eq!(PageIx::FIRST.index(), 0);
    }

    #[derive(Debug, Clone)]
    enum Op {
        Insert(i32),
        Remove(usize),
    }

    fn ops() -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec(
            prop_oneof![
                any::<i32>().prop_map(Op::Insert),
                (0usize..64).prop_map(Op::Remove),
            ],
            0..120,
        )
    }

    proptest! {
        #[test]
        fn arena_matches_model_over_insert_remove_sequences(ops in ops()) {
            let mut arena = Arena::new();
            let mut issued: Vec<(Id<i32>, i32, bool)> = Vec::new();
            for op in ops {
                match op {
                    Op::Insert(v) => issued.push((arena.insert(v), v, true)),
                    Op::Remove(i) => {
                        if let Some(entry) = issued.get_mut(i) {
                            let expected = entry.2.then_some(entry.1);
                            prop_assert_eq!(arena.remove(entry.0), expected);
                            entry.2 = false;
                        }
                    }
                }
                for &(id, v, alive) in &issued {
                    prop_assert_eq!(arena.get(id).copied(), alive.then_some(v));
                }
                prop_assert_eq!(arena.len(), issued.iter().filter(|e| e.2).count());
            }
        }
    }
}
