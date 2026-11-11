#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command_palette::chord_label;
    use crate::core::keymap::{BINDINGS, Topic, topic_of};

    /// The topic of each page, in order, without repeats.
    fn page_topics() -> Vec<Topic> {
        let mut topics: Vec<Topic> = Vec::new();
        for page in pages() {
            if topics.last() != Some(&page.topic) {
                topics.push(page.topic);
            }
        }
        topics
    }

    #[test]
    fn topics_follow_the_cheat_sheet_order() {
        // Arrange
        let names: Vec<&str> = Topic::ALL.iter().map(|t| t.name()).collect();

        // Assert
        assert_eq!(
            names,
            [
                "Tools",
                "Style",
                "Edit",
                "View",
                "Helpers",
                "Gestures",
                "Typing in a text block",
                "Typing in a shape"
            ]
        );
        assert_eq!(page_topics(), Topic::ALL);
    }

    #[test]
    fn no_page_has_more_rows_than_fit() {
        for page in pages() {
            assert!(
                page.rows().len() <= ROWS_PER_PAGE,
                "{} has {} rows",
                page.title(),
                page.rows().len()
            );
            assert!(!page.rows().is_empty(), "{} is empty", page.title());
        }
    }

    #[test]
    fn long_topic_is_split_into_numbered_parts() {
        // Arrange
        let edit: Vec<Page> = pages().into_iter().filter(|p| p.topic == Topic::Edit).collect();
        let tools: Vec<Page> = pages().into_iter().filter(|p| p.topic == Topic::Tools).collect();

        // Assert
        assert!(edit.len() >= 2, "Edit needs more than one page");
        let n = edit.len();
        for (i, page) in edit.iter().enumerate() {
            assert_eq!(page.title(), format!("Edit {}/{n}", i + 1));
        }
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].title(), "Tools");
    }

    #[test]
    fn split_parts_keep_every_row_once() {
        // Arrange
        let expected = BINDINGS
            .iter()
            .filter(|&&(_, command, _)| topic_of(command) == Topic::Edit)
            .count();

        // Act
        let shown: usize = pages()
            .iter()
            .filter(|p| p.topic == Topic::Edit)
            .map(|p| p.rows().len())
            .sum();

        // Assert
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_binding_is_on_exactly_one_page() {
        // Arrange
        let rows: Vec<(String, &str)> = pages()
            .iter()
            .flat_map(|p| p.rows())
            .map(|r| (r.keys, r.text))
            .collect();

        // Act / Assert
        for &(chord, _, description) in BINDINGS {
            let wanted = (chord_label(chord), description);
            let count = rows.iter().filter(|r| **r == wanted).count();
            assert_eq!(count, 1, "{wanted:?} is on {count} pages");
        }
    }

    #[test]
    fn gesture_and_typing_rows_have_text() {
        for page in pages() {
            if matches!(
                page.topic,
                Topic::Gestures | Topic::TypingText | Topic::TypingShape
            ) {
                assert!(!page.rows().is_empty());
                for row in page.rows() {
                    assert!(!row.keys.trim().is_empty(), "{}", page.title());
                    assert!(!row.text.trim().is_empty(), "{}", page.title());
                }
            }
        }
    }

    #[test]
    fn sheet_starts_on_the_first_page() {
        assert_eq!(Sheet::new().page(), 0);
        assert_eq!(Sheet::new().current().topic, Topic::Tools);
    }

    #[test]
    fn sheet_next_and_prev_wrap() {
        // Arrange
        let mut sheet = Sheet::new();
        let last = pages().len() - 1;

        // Act / Assert
        sheet.prev();
        assert_eq!(sheet.page(), last);
        sheet.next();
        assert_eq!(sheet.page(), 0);
        sheet.next();
        assert_eq!(sheet.page(), 1);
    }

    #[test]
    fn sheet_jump_goes_to_the_first_page_of_the_topic() {
        // Arrange
        let mut sheet = Sheet::new();

        // Act
        let moved = sheet.jump(3);

        // Assert
        assert!(moved);
        assert_eq!(sheet.current().topic, Topic::Edit);
        assert_eq!(sheet.current().part, 0);
        assert!(sheet.jump(4));
        assert_eq!(sheet.current().topic, Topic::View);
    }

    #[test]
    fn sheet_jump_without_a_topic_is_ignored() {
        // Arrange
        let mut sheet = Sheet::new();
        sheet.next();
        let before = sheet.page();

        // Act / Assert
        assert!(!sheet.jump(0));
        assert!(!sheet.jump(9));
        assert!(!sheet.jump(usize::MAX));
        assert_eq!(sheet.page(), before);
    }
}
