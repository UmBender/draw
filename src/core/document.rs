//! Ordered shape store with stable ids and atomic transactions.
//!
//! The [`Document`] is the list of shapes in z-order (index 0 is drawn first,
//! at the bottom). It is only mutated through [`Document::apply`], which takes
//! a [`Transaction`] of [`Edit`]s and either applies all of them or none
//! (ADR-0005, ADR-T06-1). The `tx_*` builders create the common transactions.

use std::collections::HashSet;
use std::fmt;

use crate::core::shape::Shape;

/// Stable identity of a shape in a [`Document`].
///
/// Allocated by [`Document::next_id`]; never reused, even after undo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ShapeId(pub u64);

impl ShapeId {
    /// The raw id value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// One reversible change to a [`Document`].
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Insert `shape` with `id` so that it ends up at `index` (`0..=len`).
    Insert {
        /// Z-order position after insertion.
        index: usize,
        /// Id of the new shape; must not be present yet.
        id: ShapeId,
        /// The shape to insert; must be finite.
        shape: Shape,
    },
    /// Remove the shape at `index`, which must be exactly `(id, shape)`.
    Remove {
        /// Z-order position of the shape.
        index: usize,
        /// Id expected at `index`.
        id: ShapeId,
        /// Shape expected at `index` (kept so the edit can be inverted).
        shape: Shape,
    },
    /// Replace the shape `id`, which must currently equal `before`, by `after`.
    Replace {
        /// Id of the shape to replace.
        id: ShapeId,
        /// Shape expected before the edit.
        before: Shape,
        /// Shape after the edit; must be finite.
        after: Shape,
    },
}

impl Edit {
    /// The edit that undoes this one.
    #[must_use]
    pub fn inverse(&self) -> Self {
        match self {
            Self::Insert { index, id, shape } => Self::Remove {
                index: *index,
                id: *id,
                shape: shape.clone(),
            },
            Self::Remove { index, id, shape } => Self::Insert {
                index: *index,
                id: *id,
                shape: shape.clone(),
            },
            Self::Replace { id, before, after } => Self::Replace {
                id: *id,
                before: after.clone(),
                after: before.clone(),
            },
        }
    }
}

/// An ordered group of edits applied atomically: one user action.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Transaction(pub Vec<Edit>);

impl Transaction {
    /// True if the transaction has no edits.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of edits.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The edits, in application order.
    #[must_use]
    pub fn edits(&self) -> &[Edit] {
        &self.0
    }

    /// The transaction that undoes this one: inverse edits in reverse order.
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self(self.0.iter().rev().map(Edit::inverse).collect())
    }
}

impl From<Vec<Edit>> for Transaction {
    fn from(edits: Vec<Edit>) -> Self {
        Self(edits)
    }
}

/// Why [`Document::apply`] rejected a transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    /// An `Insert` or `Remove` index is outside the document.
    IndexOutOfRange {
        /// The offending index.
        index: usize,
        /// Document length when the edit was tried.
        len: usize,
    },
    /// A `Replace` names an id that is not in the document.
    UnknownId(ShapeId),
    /// An `Insert` uses an id that is already in the document.
    DuplicateId(ShapeId),
    /// The document does not hold what a `Remove` or `Replace` expects.
    Mismatch(ShapeId),
    /// An inserted or replacing shape has a non-finite coordinate or width.
    NonFiniteShape(ShapeId),
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexOutOfRange { index, len } => {
                write!(f, "edit index {index} is out of range for {len} shapes")
            }
            Self::UnknownId(id) => write!(f, "shape {} is not in the document", id.0),
            Self::DuplicateId(id) => write!(f, "shape {} is already in the document", id.0),
            Self::Mismatch(id) => write!(f, "shape {} does not match the edit", id.0),
            Self::NonFiniteShape(id) => write!(f, "shape {} has non-finite geometry", id.0),
        }
    }
}

impl std::error::Error for ApplyError {}

/// The ordered shape store.
#[derive(Debug, Clone, Default)]
pub struct Document {
    /// Shapes in z-order, bottom first.
    shapes: Vec<(ShapeId, Shape)>,
    /// Ids currently in `shapes`, for O(1) duplicate checks.
    ids: HashSet<ShapeId>,
    /// Next id to hand out; only ever grows.
    next_id: u64,
}

impl Document {
    /// An empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a fresh id, larger than every id handed out or inserted so far.
    ///
    /// The counter saturates at `u64::MAX`, which is unreachable in practice.
    pub fn next_id(&mut self) -> ShapeId {
        let id = ShapeId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    /// Shapes with their ids, bottom to top.
    #[must_use]
    pub fn shapes(
        &self,
    ) -> impl DoubleEndedIterator<Item = (ShapeId, &Shape)> + ExactSizeIterator + '_ {
        self.shapes.iter().map(|(id, shape)| (*id, shape))
    }

    /// The shape with `id`, if present.
    #[must_use]
    pub fn get(&self, id: ShapeId) -> Option<&Shape> {
        self.index_of(id).map(|index| &self.shapes[index].1)
    }

    /// Z-order position of `id`, if present.
    #[must_use]
    pub fn index_of(&self, id: ShapeId) -> Option<usize> {
        if !self.ids.contains(&id) {
            return None;
        }
        self.shapes.iter().position(|(stored, _)| *stored == id)
    }

    /// Number of shapes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// True if there are no shapes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// The topmost shape satisfying `pred`, searching from the top down.
    pub fn topmost_where(&self, mut pred: impl FnMut(&Shape) -> bool) -> Option<ShapeId> {
        self.shapes
            .iter()
            .rev()
            .find(|(_, shape)| pred(shape))
            .map(|(id, _)| *id)
    }

    /// Apply every edit of `tx` in order, or none of them.
    ///
    /// # Errors
    ///
    /// Returns the [`ApplyError`] of the first invalid edit; the document
    /// (shapes and id counter) is then unchanged.
    pub fn apply(&mut self, tx: &Transaction) -> Result<(), ApplyError> {
        let saved_next_id = self.next_id;
        for (done, edit) in tx.0.iter().enumerate() {
            if let Err(err) = self.apply_edit(edit) {
                self.roll_back(&tx.0[..done]);
                self.next_id = saved_next_id;
                return Err(err);
            }
        }
        Ok(())
    }

    /// Undo `applied` (edits that just succeeded), last first.
    fn roll_back(&mut self, applied: &[Edit]) {
        for edit in applied.iter().rev() {
            // The inverse of an edit that just succeeded is valid by
            // construction (ADR-T06-1), so this cannot fail.
            let undone = self.apply_edit(&edit.inverse());
            debug_assert!(undone.is_ok(), "rollback failed: {undone:?}");
        }
    }

    /// Check and apply a single edit; on error nothing changes.
    fn apply_edit(&mut self, edit: &Edit) -> Result<(), ApplyError> {
        let len = self.shapes.len();
        match edit {
            Edit::Insert { index, id, shape } => {
                let (index, id) = (*index, *id);
                if index > len {
                    return Err(ApplyError::IndexOutOfRange { index, len });
                }
                if !shape.is_finite() {
                    return Err(ApplyError::NonFiniteShape(id));
                }
                if !self.ids.insert(id) {
                    return Err(ApplyError::DuplicateId(id));
                }
                self.shapes.insert(index, (id, shape.clone()));
                self.next_id = self.next_id.max(id.0.saturating_add(1));
            }
            Edit::Remove { index, id, shape } => {
                let index = *index;
                let Some((stored_id, stored)) = self.shapes.get(index) else {
                    return Err(ApplyError::IndexOutOfRange { index, len });
                };
                if stored_id != id || stored != shape {
                    return Err(ApplyError::Mismatch(*id));
                }
                self.shapes.remove(index);
                self.ids.remove(id);
            }
            Edit::Replace { id, before, after } => {
                let index = self.index_of(*id).ok_or(ApplyError::UnknownId(*id))?;
                if !after.is_finite() {
                    return Err(ApplyError::NonFiniteShape(*id));
                }
                let slot = &mut self.shapes[index].1;
                if *slot != *before {
                    return Err(ApplyError::Mismatch(*id));
                }
                slot.clone_from(after);
            }
        }
        Ok(())
    }
}

/// Transaction inserting `shapes` on top, in order, with freshly allocated ids.
///
/// Only allocates ids; the shape list is unchanged until the transaction is
/// applied.
pub fn tx_insert(doc: &mut Document, shapes: impl IntoIterator<Item = Shape>) -> Transaction {
    let base = doc.len();
    shapes
        .into_iter()
        .enumerate()
        .map(|(offset, shape)| Edit::Insert {
            index: base + offset,
            id: doc.next_id(),
            shape,
        })
        .collect::<Vec<_>>()
        .into()
}

/// Transaction removing the shapes `ids`; unknown and repeated ids are skipped.
///
/// Edits are ordered by descending index so each index stays valid.
#[must_use]
pub fn tx_remove(doc: &Document, ids: &[ShapeId]) -> Transaction {
    let wanted: HashSet<ShapeId> = ids.iter().copied().collect();
    removals_top_down(doc, |id| wanted.contains(&id))
}

/// Transaction replacing shape `id` by `new`; empty if `id` is unknown.
#[must_use]
pub fn tx_replace(doc: &Document, id: ShapeId, new: Shape) -> Transaction {
    doc.get(id)
        .map(|before| {
            vec![Edit::Replace {
                id,
                before: before.clone(),
                after: new,
            }]
        })
        .unwrap_or_default()
        .into()
}

/// Transaction removing every shape, top down.
#[must_use]
pub fn tx_clear(doc: &Document) -> Transaction {
    removals_top_down(doc, |_| true)
}

/// `Remove` edits for every shape whose id satisfies `keep`, top down.
fn removals_top_down(doc: &Document, mut keep: impl FnMut(ShapeId) -> bool) -> Transaction {
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, (id, _))| keep(*id))
        .map(|(index, (id, shape))| Edit::Remove {
            index,
            id: *id,
            shape: shape.clone(),
        })
        .collect::<Vec<_>>()
        .into()
}

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

        assert_eq!(
            snapshot(&doc),
            vec![(new_id, line(9.0)), (ids[1], line(5.0))]
        );
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

        assert_eq!(
            result,
            Err(ApplyError::IndexOutOfRange { index: 10, len: 3 })
        );
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

        assert_eq!(
            doc.apply(&tx),
            Err(ApplyError::IndexOutOfRange { index: 2, len: 1 })
        );
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
