//! A fixed-capacity ring that keeps the newest items (ADR-T47-1).

/// The newest `capacity` items pushed, in a buffer allocated once.
#[derive(Debug, Clone)]
pub struct Ring<T> {
    items: Vec<T>,
    /// Where the next push goes once the buffer is full; the oldest item.
    head: usize,
    capacity: usize,
}

impl<T> Ring<T> {
    /// An empty ring for `capacity` items (at least one).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            items: Vec::with_capacity(capacity),
            head: 0,
            capacity,
        }
    }

    /// Adds `item`, replacing the oldest one when full. Never allocates.
    pub fn push(&mut self, item: T) {
        if self.items.len() < self.capacity {
            self.items.push(item);
        } else {
            self.items[self.head] = item;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    /// Items held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether nothing was pushed yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Most items held.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Items the buffer has room for without allocating.
    #[must_use]
    pub fn allocated(&self) -> usize {
        self.items.capacity()
    }

    /// The items, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let (newer, older) = self.items.split_at(self.head);
        older.iter().chain(newer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::VecDeque;

    #[test]
    fn ring_keeps_newest_records_in_order() {
        // Arrange
        let mut ring = Ring::new(3);

        // Act
        for n in 1..=5 {
            ring.push(n);
        }

        // Assert
        assert_eq!(ring.iter().copied().collect::<Vec<_>>(), vec![3, 4, 5]);
        assert_eq!(ring.len(), 3);
        assert_eq!(ring.capacity(), 3);
    }

    #[test]
    fn ring_push_never_grows_capacity() {
        // Arrange
        let mut ring = Ring::new(4096);
        let allocated = ring.allocated();

        // Act
        for n in 0..20_000_u32 {
            ring.push(n);
        }

        // Assert
        assert_eq!(ring.allocated(), allocated);
        assert_eq!(ring.len(), 4096);
        assert_eq!(ring.iter().next(), Some(&(20_000 - 4096)));
    }

    #[test]
    fn ring_of_zero_capacity_keeps_one() {
        // Arrange
        let mut ring = Ring::new(0);

        // Act
        ring.push('a');
        ring.push('b');

        // Assert
        assert_eq!(ring.iter().copied().collect::<String>(), "b");
    }

    proptest! {
        #[test]
        fn ring_matches_vecdeque_model(
            capacity in 1_usize..64,
            items in proptest::collection::vec(any::<u16>(), 0..300),
        ) {
            // Arrange
            let mut ring = Ring::new(capacity);
            let mut model = VecDeque::new();

            for item in items {
                // Act
                ring.push(item);
                if model.len() == capacity {
                    model.pop_front();
                }
                model.push_back(item);

                // Assert
                prop_assert_eq!(ring.len(), model.len());
                prop_assert!(ring.iter().eq(model.iter()));
            }
        }
    }
}
