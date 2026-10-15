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
