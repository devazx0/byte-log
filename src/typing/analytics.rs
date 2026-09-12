pub struct TypingAnalytics;

impl TypingAnalytics {
    /// Calculates gross words per minute based on standard 5 keystrokes per word.
    pub fn calculate_gross_wpm(keystrokes: u64, active_typing_seconds: u64) -> f32 {
        if keystrokes == 0 || active_typing_seconds == 0 {
            return 0.0;
        }

        let minutes = active_typing_seconds as f32 / 60.0;
        let words = keystrokes as f32 / 5.0;

        (words / minutes).max(0.0)
    }

    /// Calculates behavioral accuracy estimate based on backspace error recovery.
    /// Formula: max(0.0, (1.0 - (2.0 * backspaces) / keystrokes) * 100.0)
    pub fn calculate_estimated_accuracy(keystrokes: u64, backspaces: u64) -> f32 {
        if keystrokes == 0 {
            return 100.0;
        }

        let error_ratio = (backspaces as f32 * 2.0) / keystrokes as f32;
        ((1.0 - error_ratio) * 100.0).clamp(0.0, 100.0)
    }
}
