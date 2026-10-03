//! Ordered shape store with stable ids and atomic transactions.
//!
//! The [`Document`] is the list of shapes in z-order (index 0 is drawn first,
//! at the bottom). It is only mutated through [`Document::apply`], which takes
//! a [`Transaction`] of [`Edit`]s and either applies all of them or none
//! (ADR-0005, ADR-T06-1). The `tx_*` builders create the common transactions.

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{BuildHasherDefault, Hasher};

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

/// Hashes a [`ShapeId`] with one multiplication (Fibonacci hashing). Ids are
/// small sequential integers made by this process, so there is nothing to
/// defend against, and the default `SipHash` showed in the T25 budgets
/// (ADR-T25-2).
#[derive(Debug, Clone, Copy, Default)]
pub struct IdHasher(u64);

impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_u64(u64::from(byte));
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0 ^ value).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
}

/// Hasher state for maps and sets keyed by [`ShapeId`].
pub type IdBuildHasher = BuildHasherDefault<IdHasher>;

/// A set of shape ids with the cheap [`IdHasher`].
pub type IdSet = HashSet<ShapeId, IdBuildHasher>;

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
    ids: IdSet,
    /// Next id to hand out; only ever grows.
    next_id: u64,
    /// Id → z-order position, built on the first lookup after an insert or
    /// remove (ADR-T25-1).
    positions: OnceCell<HashMap<ShapeId, usize, IdBuildHasher>>,
}

/// A stretch of a transaction that [`Document::apply`] handles in one go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    /// `Remove`s with strictly decreasing indices.
    Removes(usize),
    /// `Insert`s with strictly increasing indices.
    Inserts(usize),
    /// One edit applied on its own.
    Single,
}

impl Run {
    /// The run that starts at `edits[0]`.
    fn at(edits: &[Edit]) -> Self {
        let remove_index = |edit: &Edit| match edit {
            Edit::Remove { index, .. } => Some(*index),
            _ => None,
        };
        let insert_index = |edit: &Edit| match edit {
            Edit::Insert { index, .. } => Some(*index),
            _ => None,
        };
        let removes = run_length(edits, remove_index, |prev, next| next < prev);
        if removes > 1 {
            return Self::Removes(removes);
        }
        let inserts = run_length(edits, insert_index, |prev, next| next > prev);
        if inserts > 1 {
            return Self::Inserts(inserts);
        }
        Self::Single
    }

    /// Number of edits the run covers.
    fn len(self) -> usize {
        match self {
            Self::Removes(n) | Self::Inserts(n) => n,
            Self::Single => 1,
        }
    }
}

/// How many leading edits of `edits` have an index (`index_of` is `Some`)
/// with each consecutive pair `ordered`.
fn run_length(
    edits: &[Edit],
    index_of: impl Fn(&Edit) -> Option<usize>,
    ordered: impl Fn(usize, usize) -> bool,
) -> usize {
    let mut indices = edits.iter().map(index_of);
    let Some(Some(mut prev)) = indices.next() else {
        return 0;
    };
    let mut len = 1;
    for index in indices {
        match index {
            Some(next) if ordered(prev, next) => {
                prev = next;
                len += 1;
            }
            _ => break,
        }
    }
    len
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
        self.positions().get(&id).copied()
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
        if let Err((done, err)) = self.apply_runs(&tx.0) {
            self.roll_back(&tx.0[..done]);
            self.next_id = saved_next_id;
            return Err(err);
        }
        Ok(())
    }

    /// Applies `edits` run by run ([`Run`], ADR-T25-1). On error returns
    /// how many edits were applied before the failing run, which itself
    /// changed nothing.
    fn apply_runs(&mut self, edits: &[Edit]) -> Result<(), (usize, ApplyError)> {
        let mut done = 0;
        while done < edits.len() {
            let rest = &edits[done..];
            let run = Run::at(rest);
            let result = match run {
                Run::Removes(n) => self.apply_remove_run(&rest[..n]),
                Run::Inserts(n) => self.apply_insert_run(&rest[..n]),
                Run::Single => self.apply_edit(&rest[0]),
            };
            result.map_err(|err| (done, err))?;
            done += run.len();
        }
        Ok(())
    }

    /// The id → position index, built on first use.
    fn positions(&self) -> &HashMap<ShapeId, usize, IdBuildHasher> {
        self.positions.get_or_init(|| {
            self.shapes
                .iter()
                .enumerate()
                .map(|(index, (id, _))| (*id, index))
                .collect()
        })
    }

    /// Checks a [`Run::Removes`] run against the current shapes, in edit
    /// order, then removes all of it in one pass; on error nothing changes.
    ///
    /// Indices strictly decrease, so each one still names its shape in the
    /// document before the run, and only the first can be out of range.
    fn apply_remove_run(&mut self, run: &[Edit]) -> Result<(), ApplyError> {
        let len = self.shapes.len();
        let mut doomed = vec![false; len];
        let edits = run.iter().filter_map(|edit| match edit {
            Edit::Remove { index, id, shape } => Some((index, id, shape)),
            _ => None,
        });
        for (done, (index, id, shape)) in edits.enumerate() {
            let Some((stored_id, stored)) = self.shapes.get(*index) else {
                return Err(ApplyError::IndexOutOfRange {
                    index: *index,
                    len: len - done,
                });
            };
            if stored_id != id || stored != shape {
                return Err(ApplyError::Mismatch(*id));
            }
            doomed[*index] = true;
        }
        let mut position = 0;
        self.shapes.retain(|_| {
            let keep = !doomed[position];
            position += 1;
            keep
        });
        for edit in run {
            if let Edit::Remove { id, .. } = edit {
                self.ids.remove(id);
            }
        }
        self.positions.take();
        Ok(())
    }

    /// Checks a [`Run::Inserts`] run, in edit order, then merges all of it
    /// in one pass; on error nothing changes.
    ///
    /// Indices strictly increase, so each one is the shape's final position.
    fn apply_insert_run(&mut self, run: &[Edit]) -> Result<(), ApplyError> {
        let len = self.shapes.len();
        let mut new_ids = IdSet::with_capacity_and_hasher(run.len(), IdBuildHasher::default());
        let mut inserts = Vec::with_capacity(run.len());
        let edits = run.iter().filter_map(|edit| match edit {
            Edit::Insert { index, id, shape } => Some((index, id, shape)),
            _ => None,
        });
        for (done, (index, id, shape)) in edits.enumerate() {
            let current = len + done;
            if *index > current {
                return Err(ApplyError::IndexOutOfRange {
                    index: *index,
                    len: current,
                });
            }
            if !shape.is_finite() {
                return Err(ApplyError::NonFiniteShape(*id));
            }
            if self.ids.contains(id) || !new_ids.insert(*id) {
                return Err(ApplyError::DuplicateId(*id));
            }
            inserts.push((*index, *id, shape));
        }
        let total = len + inserts.len();
        let mut old = std::mem::take(&mut self.shapes).into_iter();
        let mut inserts = inserts.into_iter().peekable();
        let mut merged = Vec::with_capacity(total);
        while merged.len() < total {
            let next = match inserts.next_if(|(index, _, _)| *index == merged.len()) {
                Some((_, id, shape)) => Some((id, shape.clone())),
                None => old.next(),
            };
            let Some(next) = next else {
                break;
            };
            merged.push(next);
        }
        self.shapes = merged;
        for id in new_ids {
            self.ids.insert(id);
            self.next_id = self.next_id.max(id.0.saturating_add(1));
        }
        self.positions.take();
        Ok(())
    }

    /// Undo `applied` (edits that just succeeded) with their inverses, last
    /// first, through the same runs.
    fn roll_back(&mut self, applied: &[Edit]) {
        let undo: Vec<Edit> = applied.iter().rev().map(Edit::inverse).collect();
        // The inverse of an edit that just succeeded is valid by
        // construction (ADR-T06-1), so this cannot fail.
        let undone = self.apply_runs(&undo);
        debug_assert!(undone.is_ok(), "rollback failed: {undone:?}");
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
                self.positions.take();
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
                self.positions.take();
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
    let wanted: IdSet = ids.iter().copied().collect();
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

    // ---- T25 AC-1: O(1) lookup ----

    /// Asserts that `get` and `index_of` agree with a scan of `doc`, and that
    /// ids `0..probe` missing from it are not found.
    fn assert_lookups_match_scan(doc: &Document, probe: u64) {
        let shapes = snapshot(doc);
        for (pos, (id, shape)) in shapes.iter().enumerate() {
            assert_eq!(doc.index_of(*id), Some(pos), "{id:?}");
            assert_eq!(doc.get(*id), Some(shape), "{id:?}");
        }
        for raw in 0..probe {
            let id = ShapeId(raw);
            if shapes.iter().all(|(stored, _)| *stored != id) {
                assert_eq!(doc.index_of(id), None, "{id:?}");
                assert!(doc.get(id).is_none(), "{id:?}");
            }
        }
    }

    #[test]
    fn index_of_after_middle_remove_is_shifted() {
        // Arrange: look up once so the index exists.
        let (mut doc, ids) = doc_with(4);
        assert_eq!(doc.index_of(ids[2]), Some(2));

        // Act
        assert!(doc.apply(&tx_remove(&doc, &[ids[1]])).is_ok());

        // Assert
        assert_eq!(doc.index_of(ids[1]), None);
        assert_eq!(doc.index_of(ids[2]), Some(1));
        assert_eq!(doc.index_of(ids[3]), Some(2));
    }

    #[test]
    fn index_of_after_middle_insert_is_shifted() {
        // Arrange
        let (mut doc, ids) = doc_with(3);
        assert_eq!(doc.index_of(ids[1]), Some(1));
        let id = doc.next_id();
        let tx = Transaction(vec![Edit::Insert {
            index: 1,
            id,
            shape: line(9.0),
        }]);

        // Act
        assert!(doc.apply(&tx).is_ok());

        // Assert
        assert_eq!(doc.index_of(id), Some(1));
        assert_eq!(doc.index_of(ids[1]), Some(2));
        assert_eq!(doc.get(id), Some(&line(9.0)));
    }

    #[test]
    fn get_after_replace_returns_new_shape() {
        // Arrange
        let (mut doc, ids) = doc_with(3);
        assert_eq!(doc.get(ids[1]), Some(&line(1.0)));

        // Act
        assert!(doc.apply(&tx_replace(&doc, ids[1], line(7.0))).is_ok());

        // Assert
        assert_eq!(doc.get(ids[1]), Some(&line(7.0)));
        assert_eq!(doc.index_of(ids[1]), Some(1));
    }

    #[test]
    fn failed_transaction_keeps_lookups() {
        // Arrange: a valid removal followed by an invalid replace.
        let (mut doc, ids) = doc_with(4);
        assert_lookups_match_scan(&doc, 8);
        let before = snapshot(&doc);
        let mut edits = tx_remove(&doc, &[ids[0], ids[2]]).0;
        edits.push(Edit::Replace {
            id: ids[3],
            before: line(42.0),
            after: line(1.0),
        });

        // Act
        let result = doc.apply(&Transaction(edits));

        // Assert
        assert_eq!(result, Err(ApplyError::Mismatch(ids[3])));
        assert_eq!(snapshot(&doc), before);
        assert_lookups_match_scan(&doc, 8);
    }

    // ---- T25 AC-2: linear transactions ----

    /// Reference semantics: applies `tx`'s edits one by one to a plain list
    /// with the documented checks, in their documented order.
    fn model_apply(
        shapes: &[(ShapeId, Shape)],
        tx: &Transaction,
    ) -> Result<Vec<(ShapeId, Shape)>, ApplyError> {
        let mut shapes = shapes.to_vec();
        for edit in tx.edits() {
            let len = shapes.len();
            match edit {
                Edit::Insert { index, id, shape } => {
                    if *index > len {
                        return Err(ApplyError::IndexOutOfRange { index: *index, len });
                    }
                    if !shape.is_finite() {
                        return Err(ApplyError::NonFiniteShape(*id));
                    }
                    if shapes.iter().any(|(stored, _)| stored == id) {
                        return Err(ApplyError::DuplicateId(*id));
                    }
                    shapes.insert(*index, (*id, shape.clone()));
                }
                Edit::Remove { index, id, shape } => {
                    let Some((stored_id, stored)) = shapes.get(*index) else {
                        return Err(ApplyError::IndexOutOfRange { index: *index, len });
                    };
                    if stored_id != id || stored != shape {
                        return Err(ApplyError::Mismatch(*id));
                    }
                    shapes.remove(*index);
                }
                Edit::Replace { id, before, after } => {
                    let Some(pos) = shapes.iter().position(|(stored, _)| stored == id) else {
                        return Err(ApplyError::UnknownId(*id));
                    };
                    if !after.is_finite() {
                        return Err(ApplyError::NonFiniteShape(*id));
                    }
                    if shapes[pos].1 != *before {
                        return Err(ApplyError::Mismatch(*id));
                    }
                    shapes[pos].1 = after.clone();
                }
            }
        }
        Ok(shapes)
    }

    /// An edit from small random numbers: often valid against
    /// [`doc_with`], often not.
    fn raw_edit((kind, index, id, x): (u8, usize, u64, u8)) -> Edit {
        let shape = if x == 13 {
            nan_line()
        } else {
            line(f32::from(x))
        };
        match kind {
            0 => Edit::Insert {
                index,
                id: ShapeId(id),
                shape,
            },
            1 => Edit::Remove {
                index,
                id: ShapeId(id),
                shape,
            },
            _ => Edit::Replace {
                id: ShapeId(id),
                before: line(f32::from(x)),
                after: line(f32::from(x) + 0.5),
            },
        }
    }

    #[test]
    fn apply_remove_run_with_mismatch_changes_nothing() {
        // Arrange: a top-down removal run whose middle edit is stale.
        let (mut doc, ids) = doc_with(5);
        let before = snapshot(&doc);
        let mut tx = tx_remove(&doc, &[ids[0], ids[2], ids[4]]);
        if let Edit::Remove { shape, .. } = &mut tx.0[1] {
            *shape = line(99.0);
        }

        // Act
        let result = doc.apply(&tx);

        // Assert
        assert_eq!(result, Err(ApplyError::Mismatch(ids[2])));
        assert_eq!(snapshot(&doc), before);
        assert_lookups_match_scan(&doc, 8);
    }

    #[test]
    fn apply_insert_run_with_duplicate_id_changes_nothing() {
        // Arrange: undo of a removal run, with its last insert reusing an id
        // that is still in the document.
        let (mut doc, ids) = doc_with(5);
        let removal = tx_remove(&doc, &[ids[0], ids[2], ids[4]]);
        assert!(doc.apply(&removal).is_ok());
        let before = snapshot(&doc);
        let mut undo = removal.inverse();
        if let Some(Edit::Insert { id, .. }) = undo.0.last_mut() {
            *id = ids[1];
        }

        // Act
        let result = doc.apply(&undo);

        // Assert
        assert_eq!(result, Err(ApplyError::DuplicateId(ids[1])));
        assert_eq!(snapshot(&doc), before);
        assert_lookups_match_scan(&doc, 8);
    }

    proptest! {
        #[test]
        fn apply_matches_edit_by_edit_application(
            n in 0usize..12,
            mask in prop::collection::vec(any::<bool>(), 12),
            removes_first in any::<bool>(),
            raw in prop::collection::vec((0u8..3, 0usize..14, 0u64..16, 0u8..14), 0..8),
        ) {
            // Arrange: optionally a valid removal run, then random edits.
            let (doc, ids) = doc_with(n);
            let mut edits = Vec::new();
            if removes_first {
                let chosen: Vec<ShapeId> = ids
                    .iter()
                    .zip(&mask)
                    .filter(|(_, keep)| **keep)
                    .map(|(id, _)| *id)
                    .collect();
                edits.extend(tx_remove(&doc, &chosen).0);
            }
            edits.extend(raw.into_iter().map(raw_edit));
            let tx = Transaction(edits);
            let expected = model_apply(&snapshot(&doc), &tx);

            // Act
            let mut actual = doc.clone();
            let result = actual.apply(&tx);

            // Assert
            match expected {
                Ok(shapes) => {
                    prop_assert_eq!(result, Ok(()));
                    prop_assert_eq!(snapshot(&actual), shapes);
                    // Undo restores the original, like the model says.
                    prop_assert_eq!(actual.apply(&tx.inverse()), Ok(()));
                    prop_assert_eq!(snapshot(&actual), snapshot(&doc));
                }
                Err(err) => {
                    prop_assert_eq!(result, Err(err));
                    prop_assert_eq!(snapshot(&actual), snapshot(&doc));
                }
            }
            assert_lookups_match_scan(&actual, 20);
        }

        #[test]
        fn lookups_match_a_scan_after_random_transactions(
            steps in prop::collection::vec((0u8..4, prop::collection::vec(any::<bool>(), 16)), 1..12),
        ) {
            // Arrange
            let (mut doc, _) = doc_with(6);
            let mut undo: Vec<Transaction> = Vec::new();

            for (kind, mask) in steps {
                let ids: Vec<ShapeId> = doc.shapes().map(|(id, _)| id).collect();
                let chosen: Vec<ShapeId> = ids
                    .iter()
                    .zip(&mask)
                    .filter(|(_, keep)| **keep)
                    .map(|(id, _)| *id)
                    .collect();

                // Act
                let tx = match kind {
                    0 => tx_insert(&mut doc, (0..3u8).map(|i| line(f32::from(i)))),
                    1 => tx_remove(&doc, &chosen),
                    2 => chosen
                        .first()
                        .map(|id| tx_replace(&doc, *id, line(5.5)))
                        .unwrap_or_default(),
                    _ => undo.pop().map(|tx| tx.inverse()).unwrap_or_default(),
                };
                prop_assert_eq!(doc.apply(&tx), Ok(()));
                if kind != 3 {
                    undo.push(tx);
                }

                // Assert
                assert_lookups_match_scan(&doc, 64);
            }
        }
    }
}
