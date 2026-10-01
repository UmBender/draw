//! Auto-numbering of new nodes (T19).
//!
//! With numbering on, every committed [`Shape::Rect`] or [`Shape::Ellipse`]
//! gets the next number as its label (ADR-T16-1). The editor stores the
//! switch and the counter in [`Helpers`] (ADR-T16-3).
//!
//! Undo gives a number back and redo takes it again (ADR-T19-1): the editor
//! counts the shapes carrying a label before and after each step
//! ([`label_count`]) and moves the counter with [`advance`] and
//! [`roll_back`].

use crate::core::document::Document;
use crate::core::editor::Helpers;
use crate::core::shape::Shape;

/// The first number given out, and the counter value after a reset.
pub const FIRST_NUMBER: u32 = 1;

/// `shape` with the label numbering gives a newly committed shape under
/// `helpers`: `Some(next_number)` on a rectangle or ellipse while numbering
/// is on; any other shape, or numbering off, leaves `shape` unchanged.
#[must_use]
pub fn label_new(shape: Shape, helpers: &Helpers) -> Shape {
    match shape {
        Shape::Rect { .. } | Shape::Ellipse { .. } if helpers.numbering => {
            shape.with_label(Some(helpers.next_number))
        }
        _ => shape,
    }
}

/// Number of shapes in `doc` labelled `label`.
#[must_use]
pub fn label_count(doc: &Document, label: u32) -> usize {
    doc.shapes()
        .filter(|(_, shape)| shape.label() == Some(label))
        .count()
}

/// The counter after a gesture end or redo: `next + 1` (saturating) if the
/// number of shapes labelled `next` grew from `before` to `after`, else
/// `next`.
#[must_use]
pub fn advance(next: u32, before: usize, after: usize) -> u32 {
    if after > before {
        next.saturating_add(1)
    } else {
        next
    }
}

/// The counter after an undo: `next − 1` if the number of shapes labelled
/// `next − 1` shrank from `before` to `after`, else `next`; never below
/// [`FIRST_NUMBER`].
#[must_use]
pub fn roll_back(next: u32, before: usize, after: usize) -> u32 {
    let next = next.max(FIRST_NUMBER);
    if next > FIRST_NUMBER && after < before {
        next - 1
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{Document, tx_insert};
    use crate::core::geom::Vec2;
    use crate::core::palette::ColorId;
    use crate::core::shape::Style;

    const STYLE: Style = Style {
        color: ColorId::INK,
        width: 1.0,
    };

    fn ellipse() -> Shape {
        Shape::Ellipse {
            a: Vec2::ZERO,
            b: Vec2::new(20.0, 20.0),
            style: STYLE,
            fill: None,
            label: None,
        }
    }

    fn rect() -> Shape {
        Shape::Rect {
            a: Vec2::ZERO,
            b: Vec2::new(30.0, 20.0),
            style: STYLE,
            fill: None,
            label: None,
        }
    }

    fn line() -> Shape {
        Shape::Line {
            a: Vec2::ZERO,
            b: Vec2::new(30.0, 20.0),
            style: STYLE,
        }
    }

    /// Numbering on, next number `next`.
    fn on(next: u32) -> Helpers {
        Helpers {
            numbering: true,
            next_number: next,
            ..Helpers::default()
        }
    }

    // ---- AC-1 -------------------------------------------------------------

    #[test]
    fn new_circle_gets_next_number() {
        let shape = label_new(ellipse(), &on(7));

        assert_eq!(shape.label(), Some(7));
        assert!(matches!(shape, Shape::Ellipse { .. }));
    }

    #[test]
    fn rect_gets_next_number() {
        let shape = label_new(rect(), &on(3));

        assert_eq!(shape.label(), Some(3));
        assert!(matches!(shape, Shape::Rect { .. }));
    }

    #[test]
    fn line_gets_no_label() {
        let shape = label_new(line(), &on(3));

        assert_eq!(shape, line());
        assert_eq!(shape.label(), None);
    }

    #[test]
    fn numbering_off_gives_no_label() {
        let helpers = Helpers {
            numbering: false,
            next_number: 4,
            ..Helpers::default()
        };

        assert_eq!(label_new(ellipse(), &helpers), ellipse());
        assert_eq!(label_new(rect(), &helpers), rect());
    }

    // ---- AC-2 -------------------------------------------------------------

    #[test]
    fn starts_at_one() {
        assert_eq!(FIRST_NUMBER, 1);
        assert_eq!(Helpers::default().next_number, FIRST_NUMBER);
        assert_eq!(
            label_new(ellipse(), &on(Helpers::default().next_number)).label(),
            Some(1)
        );
    }

    // ---- AC-3 -------------------------------------------------------------

    #[test]
    fn label_count_counts_matches() {
        // Arrange
        let mut doc = Document::new();
        let shapes = [
            ellipse().with_label(Some(2)),
            rect().with_label(Some(2)),
            ellipse().with_label(Some(3)),
            line(),
            rect(),
        ];
        let tx = tx_insert(&mut doc, shapes);
        assert!(doc.apply(&tx).is_ok());

        // Act / Assert
        assert_eq!(label_count(&doc, 2), 2);
        assert_eq!(label_count(&doc, 3), 1);
        assert_eq!(label_count(&doc, 1), 0);
        assert_eq!(label_count(&Document::new(), 1), 0);
    }

    #[test]
    fn advance_when_count_grows() {
        assert_eq!(advance(4, 0, 1), 5);
        assert_eq!(advance(1, 1, 2), 2, "a duplicate after a reset");
        assert_eq!(advance(4, 1, 1), 4, "unchanged count");
        assert_eq!(advance(4, 2, 1), 4, "shrinking count");
    }

    #[test]
    fn advance_saturates() {
        assert_eq!(advance(u32::MAX, 0, 1), u32::MAX);
    }

    #[test]
    fn roll_back_when_count_shrinks() {
        assert_eq!(roll_back(5, 1, 0), 4);
        assert_eq!(roll_back(2, 2, 1), 1, "a duplicate after a reset");
        assert_eq!(roll_back(5, 1, 1), 5, "unchanged count");
        assert_eq!(roll_back(5, 0, 1), 5, "growing count");
    }

    #[test]
    fn roll_back_stops_at_first() {
        assert_eq!(roll_back(FIRST_NUMBER, 1, 0), FIRST_NUMBER);
        assert_eq!(roll_back(0, 1, 0), FIRST_NUMBER, "never below the start");
    }
}
