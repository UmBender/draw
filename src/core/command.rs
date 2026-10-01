//! Editor commands triggered by keys and toolbar buttons.
//!
//! Owned by T08; filled in by that task.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_default_is_pen() {
        assert_eq!(Tool::default(), Tool::Pen);
    }

    #[test]
    fn tool_all_lists_each_tool_once() {
        // Arrange
        let all = Tool::ALL;

        // Act
        let unique: std::collections::HashSet<Tool> = all.iter().copied().collect();

        // Assert
        assert_eq!(all.len(), 9);
        assert_eq!(unique.len(), all.len());
    }
}
