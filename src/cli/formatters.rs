pub struct Formatter;

impl Formatter {
    pub fn format_duration(seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}h {:02}m {:02}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {:02}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    pub fn format_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        const TB: f64 = GB * 1024.0;

        let b = bytes as f64;
        if b >= TB {
            format!("{:.2} TB", b / TB)
        } else if b >= GB {
            format!("{:.2} GB", b / GB)
        } else if b >= MB {
            format!("{:.2} MB", b / MB)
        } else if b >= KB {
            format!("{:.1} KB", b / KB)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn render_progress_bar(percentage: f32, width: usize) -> String {
        let clamped = percentage.clamp(0.0, 100.0);
        let filled = ((clamped / 100.0) * width as f32).round() as usize;
        let empty = width.saturating_sub(filled);

        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }

    pub fn format_number(n: u64) -> String {
        let s = n.to_string();
        let mut result = String::new();
        let len = s.len();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result
    }
}
