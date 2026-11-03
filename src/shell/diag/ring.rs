//! A fixed-capacity ring that keeps the newest items (ADR-T47-1).

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
