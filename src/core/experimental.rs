//! Experimental features behind the `DRAW_EXPERIMENTAL` switch (T31,
//! ADR-T31-1).
//!
//! The variable holds a list of feature names separated by commas or
//! whitespace, matched case-insensitively; unknown names are ignored. The
//! shell reads it once at start-up and hands the parsed [`Experimental`]
//! to the editor, which carries it in its helper settings.

/// Name of the environment variable listing experimental features.
pub const EXPERIMENTAL_ENV: &str = "DRAW_EXPERIMENTAL";

/// Feature name of text in rectangles, ellipses and grid cells.
pub const SHAPE_TEXT: &str = "shape-text";

/// Which experimental features are on; all off by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Experimental {
    /// Text in rectangles, ellipses and grid cells ([`SHAPE_TEXT`]).
    pub shape_text: bool,
}

impl Experimental {
    /// The features named in `value`, the value of [`EXPERIMENTAL_ENV`]
    /// (`None` when unset).
    #[must_use]
    pub fn from_env_value(value: Option<&str>) -> Self {
        let _ = value;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_value_unset_or_empty_is_all_off() {
        assert_eq!(Experimental::from_env_value(None), Experimental::default());
        assert_eq!(
            Experimental::from_env_value(Some("")),
            Experimental::default()
        );
        assert!(!Experimental::default().shape_text);
    }

    #[test]
    fn from_env_value_shape_text_turns_it_on() {
        let on = Experimental::from_env_value(Some(SHAPE_TEXT));

        assert!(on.shape_text);
    }

    #[test]
    fn from_env_value_is_a_case_insensitive_list_ignoring_unknown_names() {
        for value in [
            "Shape-Text",
            " shape-text ",
            "foo,shape-text",
            "foo shape-text bar",
            "foo, SHAPE-TEXT",
        ] {
            assert!(
                Experimental::from_env_value(Some(value)).shape_text,
                "{value:?}"
            );
        }
        for value in ["foo", "shape_text", "shapetext", "shape-text-2", "0"] {
            assert!(
                !Experimental::from_env_value(Some(value)).shape_text,
                "{value:?}"
            );
        }
    }
}
