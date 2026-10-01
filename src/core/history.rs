//! Transactional undo/redo log.
//!
//! Owned by T06; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{
        Document, Edit, ShapeId, Transaction, tx_clear, tx_insert, tx_remove, tx_replace,
    };
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use proptest::prelude::*;

    fn line(x: f32) -> Shape {
        Shape::Line {
            a: Vec2::new(x, 0.0),
            b: Vec2::new(x, 10.0),
            style: Style {
                color: ColorId::INK,
                width: 2.0,
            },
        }
    }

    fn snapshot(doc: &Document) -> Vec<(ShapeId, Shape)> {
        doc.shapes().map(|(id, s)| (id, s.clone())).collect()
    }

    fn commit_insert(history: &mut History, doc: &mut Document, x: f32) {
        let tx = tx_insert(doc, [line(x)]);
        assert!(history.commit(doc, tx).is_ok());
    }

    // ---- AC-5 ----

    #[test]
    fn undo_redo_single_insert() {
        let mut doc = Document::new();
        let mut history = History::new();
        commit_insert(&mut history, &mut doc, 1.0);
        let after = snapshot(&doc);

        assert!(history.can_undo());
        assert!(history.undo(&mut doc));
        assert!(doc.is_empty());
        assert!(history.can_redo());

        assert!(history.redo(&mut doc));
        assert_eq!(snapshot(&doc), after);
        assert_eq!(history.undo_len(), 1);
        assert_eq!(history.redo_len(), 0);
    }

    #[test]
    fn undo_redo_empty_stacks_return_false() {
        let mut doc = Document::new();
        let mut history = History::default();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert!(!history.undo(&mut doc));
        assert!(!history.redo(&mut doc));
    }

    #[test]
    fn undo_redo_refuses_document_changed_behind_its_back() {
        let mut doc = Document::new();
        let mut history = History::new();
        commit_insert(&mut history, &mut doc, 1.0);
        let stray = tx_clear(&doc);
        assert!(doc.apply(&stray).is_ok());

        assert!(!history.undo(&mut doc));
        assert_eq!(history.undo_len(), 1);
        assert!(doc.is_empty());
    }

    #[test]
    fn commit_clears_redo() {
        let mut doc = Document::new();
        let mut history = History::new();
        commit_insert(&mut history, &mut doc, 1.0);
        commit_insert(&mut history, &mut doc, 2.0);
        assert!(history.undo(&mut doc));
        assert_eq!(history.redo_len(), 1);

        commit_insert(&mut history, &mut doc, 3.0);

        assert_eq!(history.redo_len(), 0);
        assert!(!history.redo(&mut doc));
        assert_eq!(doc.len(), 2);
    }

    #[test]
    fn commit_empty_transaction_is_ignored() {
        let mut doc = Document::new();
        let mut history = History::new();
        commit_insert(&mut history, &mut doc, 1.0);
        assert!(history.undo(&mut doc));

        assert_eq!(history.commit(&mut doc, Transaction::default()), Ok(()));

        assert_eq!(history.undo_len(), 0);
        assert_eq!(history.redo_len(), 1);
    }

    #[test]
    fn commit_failing_transaction_changes_nothing() {
        let mut doc = Document::new();
        let mut history = History::new();
        commit_insert(&mut history, &mut doc, 1.0);
        let before = snapshot(&doc);
        let bad = Transaction(vec![Edit::Remove {
            index: 5,
            id: ShapeId(0),
            shape: line(0.0),
        }]);

        assert!(history.commit(&mut doc, bad).is_err());

        assert_eq!(snapshot(&doc), before);
        assert_eq!(history.undo_len(), 1);
    }

    #[test]
    fn history_capped() {
        let mut doc = Document::new();
        let mut history = History::new();

        for i in 0..(HISTORY_LIMIT + 20) {
            commit_insert(&mut history, &mut doc, i as f32);
        }
        let mut undone = 0;
        while history.undo(&mut doc) {
            undone += 1;
        }

        assert_eq!(undone, HISTORY_LIMIT);
        assert_eq!(doc.len(), 20);
    }

    #[test]
    fn history_with_limit_clamps_and_clear_empties() {
        let mut doc = Document::new();
        let mut history = History::with_limit(0);
        commit_insert(&mut history, &mut doc, 1.0);
        commit_insert(&mut history, &mut doc, 2.0);
        assert_eq!(history.undo_len(), 1);

        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(doc.len(), 2);
    }

    // ---- AC-7 ----

    #[derive(Debug, Clone)]
    enum Op {
        Insert(Vec<f32>),
        Remove(Vec<usize>),
        Replace(usize, f32),
        Clear,
    }

    fn op() -> impl Strategy<Value = Op> {
        let x = -1.0e3_f32..1.0e3_f32;
        prop_oneof![
            4 => prop::collection::vec(x.clone(), 0..4).prop_map(Op::Insert),
            2 => prop::collection::vec(0_usize..16, 0..4).prop_map(Op::Remove),
            2 => (0_usize..16, x).prop_map(|(i, x)| Op::Replace(i, x)),
            1 => Just(Op::Clear),
        ]
    }

    fn build(doc: &mut Document, op: &Op) -> Transaction {
        let ids: Vec<ShapeId> = doc.shapes().map(|(id, _)| id).collect();
        let pick = |i: usize| {
            ids.get(i % ids.len().max(1))
                .copied()
                .unwrap_or(ShapeId(u64::MAX))
        };
        match op {
            Op::Insert(xs) => tx_insert(doc, xs.iter().map(|&x| line(x))),
            Op::Remove(is) => {
                let chosen: Vec<_> = is.iter().map(|&i| pick(i)).collect();
                tx_remove(doc, &chosen)
            }
            Op::Replace(i, x) => tx_replace(doc, pick(*i), line(*x)),
            Op::Clear => tx_clear(doc),
        }
    }

    proptest! {
        #[test]
        fn undo_all_redo_all_symmetry(ops in prop::collection::vec(op(), 0..40)) {
            let mut doc = Document::new();
            let mut history = History::new();
            for op in &ops {
                let tx = build(&mut doc, op);
                prop_assert!(history.commit(&mut doc, tx).is_ok());
            }
            let final_state = snapshot(&doc);

            while history.undo(&mut doc) {}
            prop_assert!(doc.is_empty());

            while history.redo(&mut doc) {}
            prop_assert_eq!(snapshot(&doc), final_state);
        }
    }
}
