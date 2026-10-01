//! Copy, cut, paste and duplicate of selected shapes.
//!
//! The clipboard is internal to the editor (no system clipboard). `copy`
//! stores clones of the selected shapes in z-order; `paste` inserts them with
//! new ids, their joint bounds centred at the cursor, and selects them; `cut`
//! is copy then delete; `duplicate` copies the selection (leaving the
//! clipboard alone) [`DUPLICATE_OFFSET_PX`] right and down. Every command
//! that changes the document is one undo step.

use crate::core::document::{Edit, tx_insert};
use crate::core::geom::Vec2;
use crate::core::shape::Shape;
use crate::core::tools::ToolCtx;
use crate::core::tools::select::{self, prune, selected_shapes};

/// Screen offset of [`duplicate`] copies, right and down, in pixels.
pub const DUPLICATE_OFFSET_PX: f32 = 16.0;

/// Shapes copied from the document.
#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    /// Copied shapes, in z-order.
    shapes: Vec<Shape>,
}

impl Clipboard {
    /// The copied shapes, in z-order.
    #[must_use]
    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }
}

/// Copies the selected shapes. Returns whether a redraw is needed (never: the
/// document and selection are unchanged). An empty selection keeps the
/// clipboard.
pub fn copy(ctx: &mut ToolCtx<'_>) -> bool {
    let shapes = selected_clones(ctx);
    if !shapes.is_empty() {
        ctx.clipboard.shapes = shapes;
    }
    false
}

/// Copies, then deletes the selected shapes (one undo step). Returns whether
/// the document changed.
pub fn cut(ctx: &mut ToolCtx<'_>) -> bool {
    copy(ctx);
    select::delete_selection(ctx)
}

/// Pastes the clipboard centred at `ctx.cursor` and selects the copies (one
/// undo step). Returns whether the document changed.
pub fn paste(ctx: &mut ToolCtx<'_>) -> bool {
    let shapes = ctx.clipboard.shapes.clone();
    let Some(bounds) = shapes.iter().map(Shape::bounds).reduce(|a, b| a.union(&b)) else {
        return false;
    };
    let delta = ctx.camera.screen_to_world(ctx.cursor) - bounds.center();
    insert_copies(ctx, shapes, delta)
}

/// Duplicates the selection [`DUPLICATE_OFFSET_PX`] right and down (one undo
/// step) and selects the copies; the clipboard is untouched. Returns whether
/// the document changed.
pub fn duplicate(ctx: &mut ToolCtx<'_>) -> bool {
    let shapes = selected_clones(ctx);
    let offset = ctx.camera.world_len(DUPLICATE_OFFSET_PX);
    insert_copies(ctx, shapes, Vec2::new(offset, offset))
}

/// Clones of the selected shapes in z-order, after dropping stale ids.
fn selected_clones(ctx: &mut ToolCtx<'_>) -> Vec<Shape> {
    prune(ctx.selection, ctx.doc);
    selected_shapes(ctx.doc, ctx.selection)
        .into_iter()
        .map(|(_, shape)| shape.clone())
        .collect()
}

/// Inserts `shapes` translated by `delta` (world) on top as one undo step and
/// selects the copies. Returns whether the document changed; when nothing is
/// inserted (no shapes, or a non-finite result) the selection is left alone.
pub(crate) fn insert_copies(ctx: &mut ToolCtx<'_>, shapes: Vec<Shape>, delta: Vec2) -> bool {
    if shapes.is_empty() || !delta.is_finite() {
        return false;
    }
    let moved = shapes.into_iter().map(|mut shape| {
        shape.translate(delta);
        shape
    });
    let tx = tx_insert(ctx.doc, moved);
    let ids = tx
        .edits()
        .iter()
        .filter_map(|edit| match edit {
            Edit::Insert { id, .. } => Some(*id),
            Edit::Remove { .. } | Edit::Replace { .. } => None,
        })
        .collect();
    if !ctx.commit(tx) {
        return false;
    }
    *ctx.selection = ids;
    true
}

/// Shared fixture for the editing-tool tests (eraser, bucket, select,
/// clipboard): editor state without the editor, so tools can be driven
/// directly through [`ToolCtx`].
#[cfg(test)]
pub(crate) mod testkit {
    use crate::core::camera::Camera;
    use crate::core::clipboard::Clipboard;
    use crate::core::command::Tool;
    use crate::core::document::{Document, ShapeId, tx_insert};
    use crate::core::editor::DrawStyle;
    use crate::core::geom::Vec2;
    use crate::core::history::History;
    use crate::core::input::Modifiers;
    use crate::core::palette::ColorId;
    use crate::core::shape::{Shape, Style};
    use crate::core::smoothing::SmoothingLevel;
    use crate::core::tools::{Phase, Pointer, ToolCtx, ToolView};

    /// `Shift` held.
    pub(crate) const SHIFT: Modifiers = Modifiers {
        shift: true,
        ctrl: false,
        alt: false,
    };

    /// `Alt` held.
    pub(crate) const ALT: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: true,
    };

    /// The state a [`ToolCtx`] borrows.
    #[derive(Debug, Default)]
    pub(crate) struct Fixture {
        pub(crate) doc: Document,
        pub(crate) history: History,
        pub(crate) selection: Vec<ShapeId>,
        pub(crate) clipboard: Clipboard,
        pub(crate) camera: Camera,
        pub(crate) style: DrawStyle,
        pub(crate) cursor: Vec2,
    }

    impl Fixture {
        /// A fixture whose document holds `shapes` (not in the history), and
        /// their ids bottom to top.
        pub(crate) fn new(shapes: Vec<Shape>) -> (Self, Vec<ShapeId>) {
            let mut fx = Self::default();
            let tx = tx_insert(&mut fx.doc, shapes);
            assert!(fx.doc.apply(&tx).is_ok());
            let ids = fx.ids();
            (fx, ids)
        }

        /// Document ids, bottom to top.
        pub(crate) fn ids(&self) -> Vec<ShapeId> {
            self.doc.shapes().map(|(id, _)| id).collect()
        }

        /// The shape `id`, cloned; panics if it is missing.
        pub(crate) fn shape(&self, id: ShapeId) -> Shape {
            match self.doc.get(id) {
                Some(shape) => shape.clone(),
                None => panic!("shape {id:?} missing"),
            }
        }

        /// Mutable tool context for `tool`.
        pub(crate) fn ctx(&mut self, tool: Tool) -> ToolCtx<'_> {
            ToolCtx {
                doc: &mut self.doc,
                history: &mut self.history,
                selection: &mut self.selection,
                clipboard: &mut self.clipboard,
                camera: &self.camera,
                tool,
                style: self.style,
                smoothing: SmoothingLevel::default(),
                cursor: self.cursor,
            }
        }

        /// Read-only view for `tool`.
        pub(crate) fn view(&self, tool: Tool) -> ToolView<'_> {
            ToolView {
                doc: &self.doc,
                camera: &self.camera,
                selection: &self.selection,
                tool,
                style: self.style,
                smoothing: SmoothingLevel::default(),
                cursor: self.cursor,
            }
        }
    }

    fn style() -> Style {
        Style {
            color: ColorId::INK,
            width: 1.0,
        }
    }

    /// Unfilled rectangle with corner `(x, y)` and size `w × h`.
    pub(crate) fn rect(x: f32, y: f32, w: f32, h: f32) -> Shape {
        Shape::Rect {
            a: Vec2::new(x, y),
            b: Vec2::new(x + w, y + h),
            style: style(),
            fill: None,
        }
    }

    /// Rectangle filled with ink.
    pub(crate) fn filled_rect(x: f32, y: f32, w: f32, h: f32) -> Shape {
        rect(x, y, w, h).with_fill(Some(ColorId::INK))
    }

    /// Unfilled ellipse in the box with corner `(x, y)` and size `w × h`.
    pub(crate) fn ellipse(x: f32, y: f32, w: f32, h: f32) -> Shape {
        Shape::Ellipse {
            a: Vec2::new(x, y),
            b: Vec2::new(x + w, y + h),
            style: style(),
            fill: None,
        }
    }

    /// Line from `(ax, ay)` to `(bx, by)`.
    pub(crate) fn line(ax: f32, ay: f32, bx: f32, by: f32) -> Shape {
        Shape::Line {
            a: Vec2::new(ax, ay),
            b: Vec2::new(bx, by),
            style: style(),
        }
    }

    /// Pointer event at screen `(x, y)` without modifiers.
    pub(crate) fn ptr(phase: Phase, x: f32, y: f32) -> Pointer {
        ptr_with(phase, x, y, Modifiers::NONE)
    }

    /// Pointer event at screen `(x, y)` with `mods`.
    pub(crate) fn ptr_with(phase: Phase, x: f32, y: f32, mods: Modifiers) -> Pointer {
        Pointer {
            phase,
            pos: Vec2::new(x, y),
            mods,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::{Fixture, filled_rect, rect};
    use super::*;
    use crate::core::camera::Camera;
    use crate::core::command::Tool;
    use crate::core::geom::Vec2;
    use crate::core::shape::Shape;

    fn corner(shape: &Shape) -> Vec2 {
        match shape {
            Shape::Rect { a, .. } => *a,
            other => panic!("expected a rect, got {other:?}"),
        }
    }

    #[test]
    fn copy_stores_selected_in_z_order() {
        // Arrange: select top then bottom.
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 1.0, 1.0),
            rect(5.0, 0.0, 1.0, 1.0),
            rect(9.0, 0.0, 1.0, 1.0),
        ]);
        fx.selection = vec![ids[2], ids[0]];

        // Act
        copy(&mut fx.ctx(Tool::Select));

        // Assert
        assert_eq!(fx.clipboard.shapes(), &[fx.shape(ids[0]), fx.shape(ids[2])]);
        assert_eq!(fx.doc.len(), 3);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn copy_with_empty_selection_keeps_clipboard() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 1.0, 1.0)]);
        fx.selection = vec![ids[0]];
        copy(&mut fx.ctx(Tool::Select));
        fx.selection.clear();

        copy(&mut fx.ctx(Tool::Select));

        assert_eq!(fx.clipboard.shapes().len(), 1);
    }

    #[test]
    fn paste_centres_at_cursor() {
        // Arrange: bounds (-0.5, -0.5)..(10.5, 10.5), centre (5, 5).
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        fx.selection = vec![ids[0]];
        copy(&mut fx.ctx(Tool::Select));
        fx.cursor = Vec2::new(100.0, 100.0);

        // Act
        let changed = paste(&mut fx.ctx(Tool::Select));

        // Assert
        assert!(changed);
        let pasted = fx.ids()[1];
        assert!(corner(&fx.shape(pasted)).approx_eq(Vec2::new(95.0, 95.0), 1e-4));
    }

    #[test]
    fn paste_centres_at_cursor_in_world_space() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        fx.selection = vec![ids[0]];
        copy(&mut fx.ctx(Tool::Select));
        fx.camera = Camera::new(Vec2::new(10.0, 0.0), 2.0);
        fx.cursor = Vec2::new(100.0, 100.0); // world (60, 50)

        paste(&mut fx.ctx(Tool::Select));

        let pasted = fx.ids()[1];
        assert!(corner(&fx.shape(pasted)).approx_eq(Vec2::new(55.0, 45.0), 1e-4));
    }

    #[test]
    fn paste_assigns_new_ids_and_selects() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 1.0, 1.0), rect(5.0, 0.0, 1.0, 1.0)]);
        fx.selection.clone_from(&ids);
        copy(&mut fx.ctx(Tool::Select));

        paste(&mut fx.ctx(Tool::Select));

        let all = fx.ids();
        assert_eq!(all.len(), 4);
        assert_eq!(&all[..2], ids.as_slice());
        assert_eq!(fx.selection, all[2..].to_vec());
    }

    #[test]
    fn paste_empty_clipboard_is_noop() {
        let (mut fx, _) = Fixture::new(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        assert!(!paste(&mut fx.ctx(Tool::Select)));

        assert_eq!(fx.doc.len(), 1);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn paste_is_one_undo_step() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 1.0, 1.0), rect(5.0, 0.0, 1.0, 1.0)]);
        fx.selection = ids;
        copy(&mut fx.ctx(Tool::Select));

        paste(&mut fx.ctx(Tool::Select));

        assert_eq!(fx.history.undo_len(), 1);
        assert!(fx.history.undo(&mut fx.doc));
        assert_eq!(fx.doc.len(), 2);
    }

    #[test]
    fn cut_copies_and_deletes_in_one_step() {
        let (mut fx, ids) = Fixture::new(vec![
            rect(0.0, 0.0, 1.0, 1.0),
            filled_rect(5.0, 0.0, 1.0, 1.0),
        ]);
        let cut_shape = fx.shape(ids[1]);
        fx.selection = vec![ids[1]];

        let changed = cut(&mut fx.ctx(Tool::Select));

        assert!(changed);
        assert_eq!(fx.ids(), vec![ids[0]]);
        assert!(fx.selection.is_empty());
        assert_eq!(fx.clipboard.shapes(), &[cut_shape]);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn cut_with_empty_selection_is_noop() {
        let (mut fx, _) = Fixture::new(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        assert!(!cut(&mut fx.ctx(Tool::Select)));

        assert_eq!(fx.doc.len(), 1);
        assert_eq!(fx.history.undo_len(), 0);
    }

    #[test]
    fn duplicate_offsets_by_screen_pixels() {
        // Arrange: at zoom 2, 16 px is 8 world units.
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 10.0, 10.0)]);
        fx.camera = Camera::new(Vec2::ZERO, 2.0);
        fx.selection = vec![ids[0]];

        // Act
        let changed = duplicate(&mut fx.ctx(Tool::Select));

        // Assert
        assert!(changed);
        let copy_id = fx.ids()[1];
        assert_ne!(copy_id, ids[0]);
        let expected = DUPLICATE_OFFSET_PX / 2.0;
        assert!(corner(&fx.shape(copy_id)).approx_eq(Vec2::new(expected, expected), 1e-4));
        assert!(corner(&fx.shape(ids[0])).approx_eq(Vec2::ZERO, 1e-6));
        assert_eq!(fx.selection, vec![copy_id]);
        assert_eq!(fx.history.undo_len(), 1);
    }

    #[test]
    fn duplicate_leaves_clipboard() {
        let (mut fx, ids) = Fixture::new(vec![rect(0.0, 0.0, 1.0, 1.0), rect(5.0, 0.0, 1.0, 1.0)]);
        fx.selection = vec![ids[0]];
        copy(&mut fx.ctx(Tool::Select));
        fx.selection = vec![ids[1]];

        duplicate(&mut fx.ctx(Tool::Select));

        assert_eq!(fx.clipboard.shapes(), &[fx.shape(ids[0])]);
    }

    #[test]
    fn duplicate_with_empty_selection_is_noop() {
        let (mut fx, _) = Fixture::new(vec![rect(0.0, 0.0, 1.0, 1.0)]);

        assert!(!duplicate(&mut fx.ctx(Tool::Select)));

        assert_eq!(fx.doc.len(), 1);
    }
}
