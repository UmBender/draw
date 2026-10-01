//! Input model: pointer, key and scroll events in screen space.
//!
//! Owned by T08; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_default_none() {
        // Arrange / Act
        let mods = Modifiers::default();

        // Assert
        assert!(!mods.shift && !mods.ctrl && !mods.alt);
    }

    #[test]
    fn modifiers_none_is_default() {
        assert_eq!(Modifiers::NONE, Modifiers::default());
    }
}
