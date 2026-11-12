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
