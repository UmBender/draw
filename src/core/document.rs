//! Ordered shape store with stable ids and atomic transactions.
//!
//! Owned by T06; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};

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

    fn nan_line() -> Shape {
        line(f32::NAN)
    }

    /// Document with shapes `line(0)`, `line(1)`, … inserted bottom to top.
    fn doc_with(n: usize) -> (Document, Vec<ShapeId>) {
        let mut doc = Document::new();
        let tx = tx_insert(&mut doc, (0..n).map(|i| line(i as f32)));
        assert!(doc.apply(&tx).is_ok());
        let ids = doc.shapes().map(|(id, _)| id).collect();
        (doc, ids)
    }

    fn snapshot(doc: &Document) -> Vec<(ShapeId, Shape)> {
        doc.shapes().map(|(id, s)| (id, s.clone())).collect()
    }

    // ---- AC-1: ids ----

    #[test]
    fn ids_are_strictly_increasing() {
        let mut doc = Document::new();

        let a = doc.next_id();
        let b = doc.next_id();
        let c = doc.next_id();

        assert!(a < b && b < c);
    }

    #[test]
    fn ids_never_reused() {
        let mut doc = Document::new();
        let tx = tx_insert(&mut doc, [line(0.0)]);
        assert!(doc.apply(&tx).is_ok());
        let first = doc.shapes().map(|(id, _)| id).next();

        assert!(doc.apply(&tx.inverse()).is_ok());
        let next = doc.next_id();

        assert!(doc.is_empty());
        assert!(first.is_some_and(|f| next > f));
    }

    #[test]
    fn insert_with_foreign_id_bumps_counter() {
        let mut doc = Document::new();
        let foreign = ShapeId(41);
        let tx = Transaction(vec![Edit::Insert {
            index: 0,
            id: foreign,
            shape: line(0.0),
        }]);

        assert!(doc.apply(&tx).is_ok());

        assert!(doc.next_id() > foreign);
        assert_eq!(foreign.get(), 41);
    }

    // ---- AC-2: queries ----

    #[test]
    fn new_document_is_empty() {
        let doc = Document::default();

        assert!(doc.is_empty());
        assert_eq!(doc.len(), 0);
        assert_eq!(doc.shapes().count(), 0);
    }

    #[test]
    fn shapes_iterates_bottom_to_top() {
        let (doc, ids) = doc_with(3);

        let got: Vec<_> = doc.shapes().map(|(_, s)| s.clone()).collect();

        assert_eq!(doc.len(), 3);
        assert_eq!(ids.len(), 3);
        assert_eq!(got, vec![line(0.0), line(1.0), line(2.0)]);
        assert_eq!(doc.shapes().next_back().map(|(id, _)| id), Some(ids[2]));
    }

    #[test]
    fn get_known_id_returns_shape() {
        let (doc, ids) = doc_with(3);

        assert_eq!(doc.get(ids[1]), Some(&line(1.0)));
    }

    #[test]
    fn get_unknown_id_is_none() {
        let (doc, _) = doc_with(2);

        assert_eq!(doc.get(ShapeId(9_999)), None);
        assert_eq!(doc.index_of(ShapeId(9_999)), None);
    }

    #[test]
    fn index_of_reports_z_order() {
        let (doc, ids) = doc_with(3);

        assert_eq!(doc.index_of(ids[0]), Some(0));
        assert_eq!(doc.index_of(ids[2]), Some(2));
    }

    #[test]
    fn topmost_where_returns_highest_match() {
        let (doc, ids) = doc_with(4);

        let hit = doc.topmost_where(|s| matches!(s, Shape::Line { a, .. } if a.x < 2.5));

        assert_eq!(hit, Some(ids[2]));
    }

    #[test]
    fn topmost_where_no_match_is_none() {
        let (doc, _) = doc_with(3);

        assert_eq!(doc.topmost_where(Shape::is_closed), None);
    }

    // ---- AC-3: inverses ----

    #[test]
    fn edit_inverse_insert_is_remove() {
        let edit = Edit::Insert {
            index: 2,
            id: ShapeId(7),
            shape: line(1.0),
        };

        let expected = Edit::Remove {
            index: 2,
            id: ShapeId(7),
            shape: line(1.0),
        };
        assert_eq!(edit.inverse(), expected);
    }

    #[test]
    fn edit_inverse_remove_is_insert() {
        let edit = Edit::Remove {
            index: 0,
            id: ShapeId(3),
            shape: line(4.0),
        };

        let expected = Edit::Insert {
            index: 0,
            id: ShapeId(3),
            shape: line(4.0),
        };
        assert_eq!(edit.inverse(), expected);
    }

    #[test]
    fn edit_inverse_replace_swaps_before_after() {
        let edit = Edit::Replace {
            id: ShapeId(1),
            before: line(1.0),
            after: line(2.0),
        };

        let expected = Edit::Replace {
            id: ShapeId(1),
            before: line(2.0),
            after: line(1.0),
        };
        assert_eq!(edit.inverse(), expected);
    }

    #[test]
    fn edit_inverse_is_involution() {
        let edits = [
            Edit::Insert {
                index: 1,
                id: ShapeId(2),
                shape: line(0.0),
            },
            Edit::Remove {
                index: 0,
                id: ShapeId(5),
                shape: line(3.0),
            },
            Edit::Replace {
                id: ShapeId(9),
                before: line(1.0),
                after: line(6.0),
            },
        ];

        for edit in edits {
            assert_eq!(edit.inverse().inverse(), edit);
        }
    }

    #[test]
    fn transaction_inverse_reverses_order() {
        let a = Edit::Insert {
            index: 0,
            id: ShapeId(1),
            shape: line(0.0),
        };
        let b = Edit::Replace {
            id: ShapeId(1),
            before: line(0.0),
            after: line(1.0),
        };
        let tx = Transaction(vec![a.clone(), b.clone()]);

        let inv = tx.inverse();

        assert_eq!(tx.len(), 2);
        assert!(!tx.is_empty());
        assert!(Transaction::default().is_empty());
        assert_eq!(inv.edits(), &[b.inverse(), a.inverse()]);
    }

    // ---- AC-4: apply ----

    #[test]
    fn apply_insert_remove_replace() {
        let (mut doc, ids) = doc_with(2);
        let new_id = doc.next_id();
        let tx = Transaction(vec![
            Edit::Insert {
                index: 1,
                id: new_id,
                shape: line(9.0),
            },
            Edit::Remove {
                index: 0,
                id: ids[0],
                shape: line(0.0),
            },
            Edit::Replace {
                id: ids[1],
                before: line(1.0),
                after: line(5.0),
            },
        ]);

        assert_eq!(doc.apply(&tx), Ok(()));

        assert_eq!(snapshot(&doc), vec![(new_id, line(9.0)), (ids[1], line(5.0))]);
    }

    #[test]
    fn apply_is_atomic_on_error() {
        let (mut doc, ids) = doc_with(3);
        let before = snapshot(&doc);
        let new_id = doc.next_id();
        let tx = Transaction(vec![
            Edit::Replace {
                id: ids[0],
                before: line(0.0),
                after: line(7.0),
            },
            Edit::Remove {
                index: 2,
                id: ids[2],
                shape: line(2.0),
            },
            Edit::Insert {
                index: 0,
                id: new_id,
                shape: line(8.0),
            },
            Edit::Remove {
                index: 10,
                id: ids[1],
                shape: line(1.0),
            },
        ]);

        let result = doc.apply(&tx);

        assert_eq!(result, Err(ApplyError::IndexOutOfRange { index: 10, len: 3 }));
        assert_eq!(snapshot(&doc), before);
        assert!(doc.next_id() > new_id);
    }

    #[test]
    fn apply_rejects_non_finite_shape() {
        let (mut doc, ids) = doc_with(1);
        let before = snapshot(&doc);
        let new_id = doc.next_id();
        let insert = Transaction(vec![Edit::Insert {
            index: 1,
            id: new_id,
            shape: nan_line(),
        }]);
        let replace = Transaction(vec![Edit::Replace {
            id: ids[0],
            before: line(0.0),
            after: nan_line(),
        }]);

        assert_eq!(doc.apply(&insert), Err(ApplyError::NonFiniteShape(new_id)));
        assert_eq!(doc.apply(&replace), Err(ApplyError::NonFiniteShape(ids[0])));
        assert_eq!(snapshot(&doc), before);
    }

    #[test]
    fn apply_rejects_bad_index() {
        let (mut doc, _) = doc_with(1);
        let id = doc.next_id();
        let tx = Transaction(vec![Edit::Insert {
            index: 2,
            id,
            shape: line(0.0),
        }]);

        assert_eq!(doc.apply(&tx), Err(ApplyError::IndexOutOfRange { index: 2, len: 1 }));
        assert_eq!(doc.len(), 1);
    }

    #[test]
    fn apply_rejects_duplicate_id() {
        let (mut doc, ids) = doc_with(1);
        let tx = Transaction(vec![Edit::Insert {
            index: 0,
            id: ids[0],
            shape: line(3.0),
        }]);

        assert_eq!(doc.apply(&tx), Err(ApplyError::DuplicateId(ids[0])));
        assert_eq!(doc.len(), 1);
    }

    #[test]
    fn apply_rejects_unknown_id() {
        let (mut doc, _) = doc_with(1);
        let tx = Transaction(vec![Edit::Replace {
            id: ShapeId(500),
            before: line(0.0),
            after: line(1.0),
        }]);

        assert_eq!(doc.apply(&tx), Err(ApplyError::UnknownId(ShapeId(500))));
    }

    #[test]
    fn apply_rejects_mismatch() {
        let (mut doc, ids) = doc_with(2);
        let before = snapshot(&doc);
        let wrong_id = Transaction(vec![Edit::Remove {
            index: 0,
            id: ids[1],
            shape: line(1.0),
        }]);
        let wrong_shape = Transaction(vec![Edit::Remove {
            index: 0,
            id: ids[0],
            shape: line(1.0),
        }]);
        let stale_replace = Transaction(vec![Edit::Replace {
            id: ids[1],
            before: line(0.0),
            after: line(3.0),
        }]);

        assert_eq!(doc.apply(&wrong_id), Err(ApplyError::Mismatch(ids[1])));
        assert_eq!(doc.apply(&wrong_shape), Err(ApplyError::Mismatch(ids[0])));
        assert_eq!(doc.apply(&stale_replace), Err(ApplyError::Mismatch(ids[1])));
        assert_eq!(snapshot(&doc), before);
    }

    #[test]
    fn apply_then_inverse_restores_document() {
        let (mut doc, ids) = doc_with(4);
        let before = snapshot(&doc);
        let tx = tx_remove(&doc, &[ids[1], ids[3]]);

        assert!(doc.apply(&tx).is_ok());
        assert!(doc.apply(&tx.inverse()).is_ok());

        assert_eq!(snapshot(&doc), before);
    }

    #[test]
    fn apply_error_displays_message() {
        let err = ApplyError::UnknownId(ShapeId(3));

        assert!(!err.to_string().is_empty());
    }

    // ---- AC-6: builders ----

    #[test]
    fn tx_builders_insert_appends_on_top() {
        let (mut doc, ids) = doc_with(1);

        let tx = tx_insert(&mut doc, [line(5.0), line(6.0)]);
        let len_before_apply = doc.len();
        assert!(doc.apply(&tx).is_ok());

        assert_eq!(len_before_apply, 1);
        let got = snapshot(&doc);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].0, ids[0]);
        assert_eq!(got[1].1, line(5.0));
        assert_eq!(got[2].1, line(6.0));
        assert!(got[0].0 < got[1].0 && got[1].0 < got[2].0);
    }

    #[test]
    fn tx_builders_remove_skips_unknown_and_duplicates() {
        let (mut doc, ids) = doc_with(3);

        let tx = tx_remove(&doc, &[ids[0], ShapeId(777), ids[0]]);
        assert!(doc.apply(&tx).is_ok());

        assert_eq!(tx.len(), 1);
        assert_eq!(doc.len(), 2);
        assert_eq!(doc.get(ids[0]), None);
    }

    #[test]
    fn tx_builders_remove_keeps_order_of_rest() {
        let (mut doc, ids) = doc_with(5);

        let tx = tx_remove(&doc, &[ids[1], ids[3], ids[0]]);
        assert!(doc.apply(&tx).is_ok());

        let left: Vec<_> = doc.shapes().map(|(id, _)| id).collect();
        assert_eq!(left, vec![ids[2], ids[4]]);
    }

    #[test]
    fn tx_builders_replace_swaps_shape() {
        let (mut doc, ids) = doc_with(2);

        let tx = tx_replace(&doc, ids[0], line(9.0));
        assert!(doc.apply(&tx).is_ok());

        assert_eq!(doc.get(ids[0]), Some(&line(9.0)));
        assert_eq!(doc.index_of(ids[0]), Some(0));
    }

    #[test]
    fn tx_builders_replace_unknown_is_empty() {
        let (doc, _) = doc_with(2);

        assert!(tx_replace(&doc, ShapeId(404), line(1.0)).is_empty());
    }

    #[test]
    fn tx_builders_clear_empties_document() {
        let (mut doc, _) = doc_with(4);
        let before = snapshot(&doc);

        let tx = tx_clear(&doc);
        assert!(doc.apply(&tx).is_ok());
        assert!(doc.is_empty());

        assert!(doc.apply(&tx.inverse()).is_ok());
        assert_eq!(snapshot(&doc), before);
    }
}
