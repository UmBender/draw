//! Backend diagnostics events (draw patch, ADR-T47-1).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_keeps_newest_and_counts_dropped() {
        // Arrange
        let pushed = MAX_EVENTS + 10;

        // Act
        for code in 0..pushed {
            push(BackendEvent::Key {
                code: code as u32,
                state: 1,
                ctrl: false,
                alt: false,
            });
        }
        let mut out = Vec::new();
        let dropped = take_events_into(&mut out);
        let mut again = Vec::new();
        let dropped_again = take_events_into(&mut again);

        // Assert
        assert!(out.len() <= MAX_EVENTS);
        assert_eq!(dropped as usize, pushed - out.len());
        let last = out.last().map(|record| record.event);
        assert_eq!(
            last,
            Some(BackendEvent::Key {
                code: (pushed - 1) as u32,
                state: 1,
                ctrl: false,
                alt: false,
            })
        );
        assert!(again.is_empty());
        assert_eq!(dropped_again, 0);
    }
}
