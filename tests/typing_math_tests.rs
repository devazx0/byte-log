use bytelog::platform::traits::TypingInputEvent;
use bytelog::typing::{TypingAnalytics, TypingState};
use proptest::prelude::*;

#[test]
fn test_wpm_calculation_basic() {
    // 250 keystrokes in 60 seconds = 50 words / 1 min = 50 WPM
    let wpm = TypingAnalytics::calculate_gross_wpm(250, 60);
    assert!((wpm - 50.0).abs() < 0.001);

    // Zero keystrokes = 0 WPM
    assert_eq!(TypingAnalytics::calculate_gross_wpm(0, 60), 0.0);
    // Zero time = 0 WPM
    assert_eq!(TypingAnalytics::calculate_gross_wpm(250, 0), 0.0);
}

#[test]
fn test_accuracy_calculation_basic() {
    // 100 keystrokes, 0 backspaces = 100%
    let acc = TypingAnalytics::calculate_estimated_accuracy(100, 0);
    assert_eq!(acc, 100.0);

    // 100 keystrokes, 5 backspaces = (1 - (10/100)) * 100 = 90%
    let acc2 = TypingAnalytics::calculate_estimated_accuracy(100, 5);
    assert!((acc2 - 90.0).abs() < 0.001);

    // Excessive backspaces clamped to 0.0
    let acc3 = TypingAnalytics::calculate_estimated_accuracy(100, 60);
    assert_eq!(acc3, 0.0);

    // Zero keystrokes = 100.0
    assert_eq!(TypingAnalytics::calculate_estimated_accuracy(0, 0), 100.0);
}

#[test]
fn test_typing_state_flushing() {
    let mut state = TypingState::new();

    state.record_event(TypingInputEvent::StandardKey);
    state.record_event(TypingInputEvent::StandardKey);
    state.record_event(TypingInputEvent::StandardKey);
    state.record_event(TypingInputEvent::StandardKey);
    state.record_event(TypingInputEvent::Backspace);

    let slice = state
        .flush_slice(chrono::Utc::now())
        .expect("Expected slice");
    assert_eq!(slice.keystrokes, 5);
    assert_eq!(slice.backspaces, 1);
    // 5 keystrokes, 1 backspace -> accuracy = (1 - 2/5) * 100 = 60%
    assert!((slice.estimated_accuracy - 60.0).abs() < 0.1);

    // Subsequent flush should be empty
    assert!(state.flush_slice(chrono::Utc::now()).is_none());
}

proptest! {
    #[test]
    fn prop_wpm_non_negative(strokes in 0u64..1_000_000, duration in 0u64..100_000) {
        let wpm = TypingAnalytics::calculate_gross_wpm(strokes, duration);
        prop_assert!(wpm >= 0.0);
    }

    #[test]
    fn prop_accuracy_bounds(strokes in 0u64..1_000_000, backs in 0u64..1_000_000) {
        let acc = TypingAnalytics::calculate_estimated_accuracy(strokes, backs);
        prop_assert!(acc >= 0.0);
        prop_assert!(acc <= 100.0);
    }
}
