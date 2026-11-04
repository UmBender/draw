//! Transactional undo/redo log.
//!
//! [`History`] commits [`Transaction`]s to a [`Document`] and keeps them on an
//! undo stack; undo applies the inverse transaction and moves it to the redo
//! stack (ADR-0005). The undo stack is capped at [`HISTORY_LIMIT`] entries and
//! both stacks together at [`HISTORY_BYTES`] by default; the oldest entry is
//! dropped first, the newest is always kept (ADR-T49-1).

use std::collections::VecDeque;

use crate::core::document::{ApplyError, Document, Transaction};

/// Default maximum number of undoable transactions.
pub const HISTORY_LIMIT: usize = 500;

/// Default budget, in estimated heap bytes, of both stacks together
/// (ADR-T49-1).
pub const HISTORY_BYTES: usize = 64 * 1024 * 1024;

/// Undo and redo stacks of committed transactions.
#[derive(Debug, Clone)]
pub struct History {
    /// Committed transactions, oldest at the front.
    undo: VecDeque<Transaction>,
    /// Undone transactions, most recently undone last.
    redo: Vec<Transaction>,
    /// Maximum length of `undo` (at least 1).
    limit: usize,
    /// Byte budget of both stacks; the newest entry is kept even above it.
    budget: usize,
    /// Estimated heap bytes held by both stacks.
    bytes: usize,
    /// Commits, undos and redos that changed the document.
    changes: u64,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

/// Same stacks, limit and budget; the change count and the byte estimate
/// do not count (ADR-T74-1).
impl PartialEq for History {
    fn eq(&self, other: &Self) -> bool {
        self.limit == other.limit
            && self.budget == other.budget
            && self.undo == other.undo
            && self.redo == other.redo
    }
}

impl History {
    /// The undo stack (oldest first), the redo stack (most recently undone
    /// last), the limit and the byte budget (snapshots, ADR-T74-1).
    #[must_use]
    pub(crate) fn parts(&self) -> (&VecDeque<Transaction>, &[Transaction], usize, usize) {
        (&self.undo, &self.redo, self.limit, self.budget)
    }

    /// A history with these stacks; the byte estimate is recomputed and the
    /// change count starts at 0. The caller checks `1 ≤ undo.len() ≤ limit`
    /// rules (snapshots, ADR-T74-1).
    #[must_use]
    pub(crate) fn from_parts(
        undo: VecDeque<Transaction>,
        redo: Vec<Transaction>,
        limit: usize,
        budget: usize,
    ) -> Self {
        let bytes = undo.iter().chain(&redo).map(Transaction::heap_bytes).sum();
        Self {
            undo,
            redo,
            limit: limit.max(1),
            budget,
            bytes,
            changes: 0,
        }
    }

    /// Empty history keeping up to [`HISTORY_LIMIT`] transactions.
    #[must_use]
    pub fn new() -> Self {
        Self::with_limit(HISTORY_LIMIT)
    }

    /// Empty history keeping up to `limit` transactions (at least 1)
    /// within [`HISTORY_BYTES`].
    #[must_use]
    pub fn with_limit(limit: usize) -> Self {
        Self::with_limits(limit, HISTORY_BYTES)
    }

    /// Empty history keeping up to `limit` transactions (at least 1) and,
    /// beyond the newest one, up to `budget` estimated heap bytes.
    #[must_use]
    pub fn with_limits(limit: usize, budget: usize) -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            limit: limit.max(1),
            budget,
            bytes: 0,
            changes: 0,
        }
    }

    /// Apply `tx` to `doc` and record it; clears the redo stack.
    ///
    /// An empty transaction is ignored (nothing recorded, redo kept).
    ///
    /// # Errors
    ///
    /// Returns the [`ApplyError`] from [`Document::apply`]; `doc` and the
    /// history are then unchanged.
    pub fn commit(&mut self, doc: &mut Document, tx: Transaction) -> Result<(), ApplyError> {
        if tx.is_empty() {
            return Ok(());
        }
        doc.apply(&tx)?;
        let dropped: usize = self.redo.drain(..).map(|tx| tx.heap_bytes()).sum();
        self.bytes = self.bytes.saturating_sub(dropped);
        self.bytes += tx.heap_bytes();
        self.changes += 1;
        self.push_undo(tx);
        Ok(())
    }

    /// Undo the latest transaction. False if there is none, or if `doc` no
    /// longer matches it (the stacks are then unchanged).
    pub fn undo(&mut self, doc: &mut Document) -> bool {
        let Some(tx) = self.undo.pop_back() else {
            return false;
        };
        if doc.apply(&tx.inverse()).is_ok() {
            self.redo.push(tx);
            self.changes += 1;
            true
        } else {
            self.undo.push_back(tx);
            false
        }
    }

    /// Redo the latest undone transaction. False if there is none, or if `doc`
    /// no longer matches it (the stacks are then unchanged).
    pub fn redo(&mut self, doc: &mut Document) -> bool {
        let Some(tx) = self.redo.pop() else {
            return false;
        };
        if doc.apply(&tx).is_ok() {
            self.changes += 1;
            self.push_undo(tx);
            true
        } else {
            self.redo.push(tx);
            false
        }
    }

    /// True if [`History::undo`] has something to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// True if [`History::redo`] has something to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Number of undoable transactions.
    #[must_use]
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Number of redoable transactions.
    #[must_use]
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    /// Estimated heap bytes held by both stacks.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// Number of commits, undos and redos that changed the document; the
    /// editor bumps its document revision when this moves (ADR-T49-1).
    #[must_use]
    pub fn changes(&self) -> u64 {
        self.changes
    }

    /// Forget both stacks (the document is untouched).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.bytes = 0;
    }

    /// Push onto the undo stack, whose bytes `self.bytes` already counts,
    /// then drop the oldest entries beyond the entry limit or, keeping the
    /// newest one, beyond the byte budget (ADR-T49-1).
    fn push_undo(&mut self, tx: Transaction) {
        self.undo.push_back(tx);
        while self.undo.len() > self.limit || (self.bytes > self.budget && self.undo.len() > 1) {
            if let Some(old) = self.undo.pop_front() {
                self.bytes = self.bytes.saturating_sub(old.heap_bytes());
            }
        }
    }
}

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

    // ---- T49 AC-2, AC-3 byte budget and change count -------------------------

    fn stroke(n: usize) -> Shape {
        Shape::Stroke {
            points: vec![Vec2::new(1.0, 2.0); n.max(1)],
            widths: None,
            style: Style {
                color: ColorId::INK,
                width: 2.0,
            },
        }
    }

    /// Commits one stroke of `n` points; returns the transaction's bytes.
    fn commit_stroke(history: &mut History, doc: &mut Document, n: usize) -> usize {
        let tx = tx_insert(doc, [stroke(n)]);
        let bytes = tx.heap_bytes();
        assert!(history.commit(doc, tx).is_ok());
        bytes
    }

    /// Bytes of a one-stroke transaction of `n` points.
    fn stroke_tx_bytes(n: usize) -> usize {
        tx_insert(&mut Document::new(), [stroke(n)]).heap_bytes()
    }

    /// Bytes held by both stacks, recomputed from scratch.
    fn held(history: &History) -> usize {
        history
            .undo
            .iter()
            .chain(&history.redo)
            .map(Transaction::heap_bytes)
            .sum()
    }

    #[test]
    fn history_over_byte_budget_drops_oldest() {
        // Arrange
        let one = stroke_tx_bytes(100);
        let mut doc = Document::new();
        let mut history = History::with_limits(HISTORY_LIMIT, one * 3);

        // Act
        for _ in 0..5 {
            commit_stroke(&mut history, &mut doc, 100);
        }

        // Assert
        assert_eq!(history.undo_len(), 3);
        assert!(history.bytes() <= one * 3);
        while history.undo(&mut doc) {}
        assert_eq!(doc.len(), 2);
    }

    #[test]
    fn history_keeps_newest_entry_over_budget() {
        // Arrange
        let mut doc = Document::new();
        let mut history = History::with_limits(HISTORY_LIMIT, 10);

        // Act
        commit_stroke(&mut history, &mut doc, 100);
        let newest = commit_stroke(&mut history, &mut doc, 200);

        // Assert
        assert_eq!(history.undo_len(), 1);
        assert_eq!(history.bytes(), newest);
        assert!(history.undo(&mut doc));
        assert_eq!(doc.len(), 1);
        assert!(history.redo(&mut doc));
        assert_eq!(doc.len(), 2);
    }

    #[test]
    fn history_bytes_follow_commit_undo_redo_and_clear() {
        // Arrange
        let mut doc = Document::new();
        let mut history = History::new();
        assert_eq!(history.bytes(), 0);

        // Act / Assert
        let a = commit_stroke(&mut history, &mut doc, 10);
        let b = commit_stroke(&mut history, &mut doc, 20);
        assert_eq!(history.bytes(), a + b);
        assert!(history.undo(&mut doc));
        assert_eq!(history.bytes(), a + b);
        assert!(history.redo(&mut doc));
        assert_eq!(history.bytes(), a + b);
        assert!(history.undo(&mut doc));
        let c = commit_stroke(&mut history, &mut doc, 30);
        assert_eq!(history.bytes(), a + c);
        history.clear();
        assert_eq!(history.bytes(), 0);
    }

    #[test]
    fn history_changes_count_every_document_change() {
        // Arrange
        let mut doc = Document::new();
        let mut history = History::new();
        assert_eq!(history.changes(), 0);

        // Act / Assert: changes.
        commit_stroke(&mut history, &mut doc, 3);
        assert_eq!(history.changes(), 1);
        assert!(history.undo(&mut doc));
        assert_eq!(history.changes(), 2);
        assert!(history.redo(&mut doc));
        assert_eq!(history.changes(), 3);

        // Act / Assert: no change.
        assert_eq!(history.commit(&mut doc, Transaction::default()), Ok(()));
        let bad = Transaction(vec![Edit::Remove {
            index: 9,
            id: ShapeId(0),
            shape: line(0.0),
        }]);
        assert!(history.commit(&mut doc, bad).is_err());
        assert!(!history.redo(&mut doc));
        assert!(history.undo(&mut doc));
        assert!(!history.undo(&mut doc));
        assert_eq!(history.changes(), 4);
    }

    #[derive(Debug, Clone)]
    enum Step {
        Commit(usize),
        Undo,
        Redo,
    }

    fn step() -> impl Strategy<Value = Step> {
        prop_oneof![
            4 => (1_usize..300).prop_map(Step::Commit),
            2 => Just(Step::Undo),
            1 => Just(Step::Redo),
        ]
    }

    proptest! {
        #[test]
        fn history_bytes_stay_exact_and_within_budget(
            steps in prop::collection::vec(step(), 0..60),
            budget in 0_usize..8_000,
        ) {
            let mut doc = Document::new();
            let mut history = History::with_limits(8, budget);
            for step in &steps {
                match step {
                    Step::Commit(n) => {
                        commit_stroke(&mut history, &mut doc, *n);
                    }
                    Step::Undo => {
                        history.undo(&mut doc);
                    }
                    Step::Redo => {
                        history.redo(&mut doc);
                    }
                }
                prop_assert_eq!(history.bytes(), held(&history));
                prop_assert!(history.undo_len() <= 8);
                prop_assert!(
                    history.bytes() <= budget
                        || history.undo_len() + history.redo_len() == 1
                );
            }
        }
    }
}
