use crate::platform::traits::TypingInputEvent;
use crate::typing::analytics::TypingAnalytics;
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TypingSlice {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub active_typing_seconds: u64,
    pub keystrokes: u64,
    pub backspaces: u64,
    pub enters: u64,
    pub modifiers: u64,
    pub estimated_wpm: f32,
    pub estimated_accuracy: f32,
}

pub struct TypingState {
    keystrokes: AtomicU64,
    backspaces: AtomicU64,
    enters: AtomicU64,
    modifiers: AtomicU64,
    slice_started_at: DateTime<Utc>,
    last_event_instant: Option<Instant>,
    active_seconds_accumulator: AtomicU64,
}

impl Default for TypingState {
    fn default() -> Self {
        Self::new()
    }
}

impl TypingState {
    pub fn new() -> Self {
        Self {
            keystrokes: AtomicU64::new(0),
            backspaces: AtomicU64::new(0),
            enters: AtomicU64::new(0),
            modifiers: AtomicU64::new(0),
            slice_started_at: Utc::now(),
            last_event_instant: None,
            active_seconds_accumulator: AtomicU64::new(0),
        }
    }

    pub fn record_event(&mut self, event: TypingInputEvent) {
        let now = Instant::now();

        if let Some(last) = self.last_event_instant {
            let diff = now.duration_since(last).as_secs();
            // If events occur within 3 seconds of each other, count towards active typing duration
            if diff <= 3 {
                self.active_seconds_accumulator
                    .fetch_add(diff.max(1), Ordering::Relaxed);
            } else {
                self.active_seconds_accumulator
                    .fetch_add(1, Ordering::Relaxed);
            }
        } else {
            self.active_seconds_accumulator
                .fetch_add(1, Ordering::Relaxed);
        }

        self.last_event_instant = Some(now);

        match event {
            TypingInputEvent::StandardKey => {
                self.keystrokes.fetch_add(1, Ordering::Relaxed);
            }
            TypingInputEvent::Backspace => {
                self.keystrokes.fetch_add(1, Ordering::Relaxed);
                self.backspaces.fetch_add(1, Ordering::Relaxed);
            }
            TypingInputEvent::Enter => {
                self.keystrokes.fetch_add(1, Ordering::Relaxed);
                self.enters.fetch_add(1, Ordering::Relaxed);
            }
            TypingInputEvent::Modifier => {
                self.modifiers.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn flush_slice(&mut self, now: DateTime<Utc>) -> Option<TypingSlice> {
        let strokes = self.keystrokes.swap(0, Ordering::SeqCst);
        let backspaces = self.backspaces.swap(0, Ordering::SeqCst);
        let enters = self.enters.swap(0, Ordering::SeqCst);
        let modifiers = self.modifiers.swap(0, Ordering::SeqCst);
        let active_secs = self.active_seconds_accumulator.swap(0, Ordering::SeqCst);

        let started_at = self.slice_started_at;
        self.slice_started_at = now;
        self.last_event_instant = None;

        if strokes == 0 && modifiers == 0 {
            return None;
        }

        let wpm = TypingAnalytics::calculate_gross_wpm(strokes, active_secs);
        let accuracy = TypingAnalytics::calculate_estimated_accuracy(strokes, backspaces);

        Some(TypingSlice {
            started_at,
            ended_at: now,
            active_typing_seconds: active_secs,
            keystrokes: strokes,
            backspaces,
            enters,
            modifiers,
            estimated_wpm: wpm,
            estimated_accuracy: accuracy,
        })
    }
}
