//! Anomaly detectors and the per-topic line limiter (ADR-T47-1).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::diag::event::{Anomaly, Event};
    use macroquad::input::KeyCode;
    use macroquad::miniquad::diag::Device;
    use std::time::Duration;

    fn micros(duration: Duration) -> u64 {
        duration.as_micros() as u64
    }

    #[test]
    fn key_watch_held_key_pressed_again_is_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();

        // Act
        let first = watch.key_down(KeyCode::A, false);
        let second = watch.key_down(KeyCode::A, false);

        // Assert
        assert_eq!(first, None);
        assert_eq!(second, Some(Anomaly::HeldKeyPressedAgain));
    }

    #[test]
    fn key_watch_repeat_is_not_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);

        // Act / Assert
        for _ in 0..3 {
            assert_eq!(watch.key_down(KeyCode::A, true), None);
        }
    }

    #[test]
    fn key_watch_release_then_press_is_not_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);
        watch.key_down(KeyCode::B, false);

        // Act
        watch.key_up(KeyCode::A);
        let again = watch.key_down(KeyCode::A, false);

        // Assert
        assert_eq!(again, None);
    }

    #[test]
    fn key_watch_two_chars_per_press_is_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);

        // Act
        let first = watch.char(false);
        let second = watch.char(false);
        let third = watch.char(false);

        // Assert
        assert_eq!(first, None);
        assert_eq!(second, Some(Anomaly::CharsPerPress));
        assert_eq!(third, None, "one anomaly per press");
    }

    #[test]
    fn key_watch_repeat_chars_are_not_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);
        assert_eq!(watch.char(false), None);

        // Act
        watch.key_down(KeyCode::A, true);
        let repeats = [watch.char(true), watch.char(true)];
        watch.key_down(KeyCode::B, false);
        let next = watch.char(false);

        // Assert
        assert_eq!(repeats, [None, None]);
        assert_eq!(next, None);
    }

    #[test]
    fn key_watch_focus_change_forgets_held_keys() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);

        // Act
        watch.focus_changed();
        let again = watch.key_down(KeyCode::A, false);

        // Assert
        assert_eq!(again, None);
    }

    #[test]
    fn idle_watch_reports_gap_over_five_minutes() {
        // Arrange
        let mut watch = IdleWatch::new();
        let gap = IDLE_GAP + Duration::from_secs(1);

        // Act
        let first = watch.input(0);
        let after = watch.input(micros(gap));

        // Assert
        assert_eq!(IDLE_GAP, Duration::from_secs(300));
        assert_eq!(first, None);
        assert_eq!(after, Some(gap));
    }

    #[test]
    fn idle_watch_short_gap_is_quiet() {
        // Arrange
        let mut watch = IdleWatch::new();
        watch.input(0);

        // Act / Assert
        assert_eq!(watch.input(micros(Duration::from_secs(200))), None);
        assert_eq!(watch.input(micros(Duration::from_secs(400))), None);
        assert_eq!(watch.input(micros(IDLE_GAP)), None, "earlier input");
    }

    #[test]
    fn backend_watch_repeat_burst_is_anomaly() {
        // Arrange
        let mut watch = BackendWatch::new();

        // Act
        let normal = watch.event(&Event::RepeatTimer { expirations: 2 });
        let burst = watch.event(&Event::RepeatTimer { expirations: 3 });

        // Assert
        assert_eq!(normal, None);
        assert_eq!(burst, Some(Anomaly::RepeatBurst));
    }

    #[test]
    fn backend_watch_second_keyboard_is_anomaly() {
        // Arrange
        let mut watch = BackendWatch::new();
        watch.event(&Event::DeviceCreated(Device::Keyboard));
        watch.event(&Event::DeviceReleased(Device::Keyboard));

        // Act
        let again = watch.event(&Event::DeviceCreated(Device::Keyboard));

        // Assert
        assert_eq!(again, Some(Anomaly::KeyboardRecreated));
    }

    #[test]
    fn backend_watch_first_keyboard_is_quiet() {
        // Arrange
        let mut watch = BackendWatch::new();

        // Act
        let pointer = watch.event(&Event::DeviceCreated(Device::Pointer));
        let keyboard = watch.event(&Event::DeviceCreated(Device::Keyboard));

        // Assert
        assert_eq!((pointer, keyboard), (None, None));
    }

    #[test]
    fn backend_watch_dropped_events_are_overflow() {
        // Arrange
        let mut watch = BackendWatch::new();

        // Act / Assert
        assert_eq!(
            watch.event(&Event::BackendDropped { count: 5 }),
            Some(Anomaly::BackendOverflow)
        );
    }

    #[test]
    fn limiter_allows_ten_then_one_notice() {
        // Arrange
        let mut limiter = Limiter::new();
        let topic = Topic::SlowFrame;

        // Act
        let verdicts: Vec<_> = (0..12).map(|_| limiter.check(topic)).collect();
        let other = limiter.check(Topic::Anomaly(Anomaly::RepeatBurst));

        // Assert
        assert_eq!(LINES_PER_TOPIC, 10);
        assert!(verdicts[..10].iter().all(|v| *v == Verdict::Write));
        assert_eq!(verdicts[10], Verdict::Notice);
        assert_eq!(verdicts[11], Verdict::Skip);
        assert_eq!(other, Verdict::Write, "topics are counted apart");
    }
}
