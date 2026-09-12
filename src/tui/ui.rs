use crate::cli::formatters::Formatter;
use crate::tui::app::{App, TimeHorizon};
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Row, Sparkline, Table},
    Frame,
};

// Catppuccin Mocha Palette Hex Colors
const CATPPUCCIN_BASE: Color = Color::Rgb(30, 30, 46);
const CATPPUCCIN_TEXT: Color = Color::Rgb(205, 214, 244);
const CATPPUCCIN_GREEN: Color = Color::Rgb(166, 227, 161);
const CATPPUCCIN_YELLOW: Color = Color::Rgb(249, 226, 175);
const CATPPUCCIN_RED: Color = Color::Rgb(243, 139, 168);
const CATPPUCCIN_CYAN: Color = Color::Rgb(137, 220, 235);
const CATPPUCCIN_LAVENDER: Color = Color::Rgb(180, 190, 254);

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Guard against small terminal size (responsive 80x24 minimum)
    if area.width < 60 || area.height < 18 {
        let warn_msg = Paragraph::new("Terminal too small. Please expand window (Min: 80x24).")
            .style(Style::default().fg(CATPPUCCIN_RED).bg(CATPPUCCIN_BASE));
        f.render_widget(warn_msg, area);
        return;
    }

    // Main layout: Header, Body, Sparkline, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(12),   // Body
            Constraint::Length(6), // Lower Sparkline
            Constraint::Length(1), // Footer Keybindings
        ])
        .split(area);

    draw_header(f, chunks[0]);
    draw_body(f, app, chunks[1]);
    draw_sparkline(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);

    if app.category_modal.is_some() {
        draw_category_modal(f, app, area);
    }
}

fn draw_header(f: &mut Frame, area: Rect) {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let title_line = Line::from(vec![
        Span::styled(
            " BYTELOG ",
            Style::default()
                .fg(CATPPUCCIN_BASE)
                .bg(CATPPUCCIN_LAVENDER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" v{} ", env!("CARGO_PKG_VERSION")),
            Style::default().fg(CATPPUCCIN_TEXT),
        ),
        Span::styled(
            " ● LIVE ",
            Style::default()
                .fg(CATPPUCCIN_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(area.width.saturating_sub(48) as usize)),
        Span::styled(now, Style::default().fg(CATPPUCCIN_CYAN)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CATPPUCCIN_LAVENDER));

    let p = Paragraph::new(title_line).block(block);
    f.render_widget(p, area);
}

fn draw_body(f: &mut Frame, app: &mut App, area: Rect) {
    let body_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    draw_hardware_pane(f, app, body_cols[0]);
    draw_activity_pane(f, app, body_cols[1]);
}

fn draw_hardware_pane(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" LIVE HARDWARE TELEMETRY ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CATPPUCCIN_CYAN));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // CPU Gauge
            Constraint::Length(2), // RAM Gauge
            Constraint::Min(4),    // Net, Disk, Battery, Temp
        ])
        .split(inner);

    if let Some(ref snap) = app.live_snapshot {
        // CPU Gauge
        let cpu_pct = snap.cpu_utilization.clamp(0.0, 100.0);
        let cpu_color = if cpu_pct > 85.0 {
            CATPPUCCIN_RED
        } else if cpu_pct > 60.0 {
            CATPPUCCIN_YELLOW
        } else {
            CATPPUCCIN_GREEN
        };

        let cpu_gauge = Gauge::default()
            .block(Block::default().title(format!("CPU Package: {:.1}%", cpu_pct)))
            .gauge_style(Style::default().fg(cpu_color).bg(CATPPUCCIN_BASE))
            .percent(cpu_pct.round() as u16);
        f.render_widget(cpu_gauge, rows[0]);

        // Memory Gauge
        let mem_used = snap.memory_used_bytes;
        let mem_tot = snap.memory_total_bytes.max(1);
        let mem_pct = ((mem_used as f64 / mem_tot as f64) * 100.0).clamp(0.0, 100.0) as u16;

        let mem_gauge = Gauge::default()
            .block(Block::default().title(format!(
                "RAM: {} / {} ({}%)",
                Formatter::format_bytes(mem_used),
                Formatter::format_bytes(mem_tot),
                mem_pct
            )))
            .gauge_style(Style::default().fg(CATPPUCCIN_CYAN).bg(CATPPUCCIN_BASE))
            .percent(mem_pct);
        f.render_widget(mem_gauge, rows[1]);

        // Net, Disk, Battery, Temp
        let bat_str = if let Some(b) = snap.battery_percent {
            let chg = if snap.battery_charging.unwrap_or(false) {
                "Charging"
            } else {
                "Discharging"
            };
            format!("{:.0}% ({})", b, chg)
        } else {
            "AC Desktop / None".to_string()
        };

        let temp_str = if let Some(t) = snap.cpu_temp_celsius {
            format!("{:.1}°C", t)
        } else {
            "N/A".to_string()
        };

        let hw_details = vec![
            Line::from(vec![
                Span::styled("Network I/O:   ", Style::default().fg(CATPPUCCIN_LAVENDER)),
                Span::raw(format!(
                    "↓ {}/s   ↑ {}/s",
                    Formatter::format_bytes(snap.net_rx_bytes_sec),
                    Formatter::format_bytes(snap.net_tx_bytes_sec)
                )),
            ]),
            Line::from(vec![
                Span::styled("Disk I/O:      ", Style::default().fg(CATPPUCCIN_LAVENDER)),
                Span::raw(format!(
                    "R: {}/s   W: {}/s",
                    Formatter::format_bytes(snap.disk_read_bytes_sec),
                    Formatter::format_bytes(snap.disk_write_bytes_sec)
                )),
            ]),
            Line::from(vec![
                Span::styled("Battery:       ", Style::default().fg(CATPPUCCIN_LAVENDER)),
                Span::raw(bat_str),
            ]),
            Line::from(vec![
                Span::styled("Temperature:   ", Style::default().fg(CATPPUCCIN_LAVENDER)),
                Span::raw(temp_str),
            ]),
        ];

        let p = Paragraph::new(hw_details);
        f.render_widget(p, rows[2]);
    }
}

fn draw_activity_pane(f: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Top Apps Table
    let horizon_str = match app.horizon {
        TimeHorizon::Today => "TODAY",
        TimeHorizon::Week => "LAST 7 DAYS",
        TimeHorizon::Month => "LAST 30 DAYS",
    };

    let apps_block = Block::default()
        .title(format!(" APPLICATION USAGE ({}) ", horizon_str))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CATPPUCCIN_LAVENDER));

    let rows_data: Vec<Row> = app
        .top_apps
        .iter()
        .skip(app.app_scroll_index)
        .take(5)
        .map(|a| {
            Row::new(vec![
                a.executable_name.clone(),
                Formatter::format_duration(a.active_seconds),
                format!("{:.1}%", a.percentage),
            ])
        })
        .collect();

    let table = Table::new(
        rows_data,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["APP", "TIME", "%"]).style(
            Style::default()
                .fg(CATPPUCCIN_CYAN)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(apps_block);

    f.render_widget(table, rows[0]);

    // Typing Ergonomics Pane
    let typing_block = Block::default()
        .title(" TYPING ERGONOMICS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CATPPUCCIN_GREEN));

    let (strokes, wpm, acc) = if let Some(ref s) = app.summary_stats {
        (s.total_keystrokes, s.average_wpm, s.estimated_accuracy)
    } else {
        (0, 0.0, 100.0)
    };

    let typing_lines = vec![
        Line::from(vec![
            Span::styled("Total Keystrokes:   ", Style::default().fg(CATPPUCCIN_TEXT)),
            Span::styled(
                format!("{} strokes", strokes),
                Style::default()
                    .fg(CATPPUCCIN_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Average Speed:      ", Style::default().fg(CATPPUCCIN_TEXT)),
            Span::styled(
                format!("{:.0} WPM", wpm),
                Style::default()
                    .fg(CATPPUCCIN_CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Behavioral Accuracy:", Style::default().fg(CATPPUCCIN_TEXT)),
            Span::styled(
                format!("{:.1}%", acc),
                Style::default().fg(CATPPUCCIN_YELLOW),
            ),
        ]),
    ];

    let tp = Paragraph::new(typing_lines).block(typing_block);
    f.render_widget(tp, rows[1]);
}

fn draw_sparkline(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" HISTORICAL CPU UTILIZATION (PAST 60 SECONDS) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CATPPUCCIN_YELLOW));

    let sparkline = Sparkline::default()
        .block(block)
        .data(&app.cpu_history)
        .style(Style::default().fg(CATPPUCCIN_GREEN));

    f.render_widget(sparkline, area);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let keys = Line::from(vec![
        Span::styled(
            "[1] ",
            Style::default()
                .fg(CATPPUCCIN_CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Today  "),
        Span::styled(
            "[2] ",
            Style::default()
                .fg(CATPPUCCIN_CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Week  "),
        Span::styled(
            "[3] ",
            Style::default()
                .fg(CATPPUCCIN_CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Month  "),
        Span::styled(
            "[r] ",
            Style::default()
                .fg(CATPPUCCIN_YELLOW)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Refresh  "),
        Span::styled(
            "[j/k] ",
            Style::default()
                .fg(CATPPUCCIN_YELLOW)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Scroll Apps  "),
        Span::styled(
            "[c] ",
            Style::default()
                .fg(CATPPUCCIN_YELLOW)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Categorize  "),
        Span::styled(
            "[q] ",
            Style::default()
                .fg(CATPPUCCIN_RED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Quit"),
    ]);

    let p = Paragraph::new(keys).style(Style::default().bg(CATPPUCCIN_BASE).fg(CATPPUCCIN_TEXT));
    f.render_widget(p, area);
}

fn draw_category_modal(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref modal) = app.category_modal {
        let popup_width = 58.min(area.width.saturating_sub(4));
        let popup_height = 13.min(area.height.saturating_sub(2));

        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_rect = Rect::new(x, y, popup_width, popup_height);

        f.render_widget(Clear, popup_rect);

        let title = format!(" RECLASSIFY APPLICATION: {} ", modal.app_name);
        let block = Block::default()
            .title(Span::styled(
                title,
                Style::default().fg(CATPPUCCIN_YELLOW).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(CATPPUCCIN_LAVENDER))
            .style(Style::default().bg(CATPPUCCIN_BASE));

        let inner = block.inner(popup_rect);
        f.render_widget(block, popup_rect);

        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::raw(" Current Category: "),
            Span::styled(
                &modal.current_category,
                Style::default().fg(CATPPUCCIN_CYAN).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::raw(""));

        // 2-column layout for categories 1..8
        let pairs = [
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        for (left_idx, right_idx) in pairs {
            let left_cat = crate::tui::app::CATEGORIES[left_idx];
            let right_cat = crate::tui::app::CATEGORIES[right_idx];
            let left_curr = left_cat.eq_ignore_ascii_case(&modal.current_category);
            let right_curr = right_cat.eq_ignore_ascii_case(&modal.current_category);

            let l_style = if left_curr {
                Style::default().fg(CATPPUCCIN_GREEN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(CATPPUCCIN_TEXT)
            };
            let r_style = if right_curr {
                Style::default().fg(CATPPUCCIN_GREEN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(CATPPUCCIN_TEXT)
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" [{}] ", left_idx + 1), Style::default().fg(CATPPUCCIN_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:<15}", left_cat), l_style),
                Span::styled(format!(" [{}] ", right_idx + 1), Style::default().fg(CATPPUCCIN_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:<15}", right_cat), r_style),
            ]));
        }

        // 9th category
        let cat9 = crate::tui::app::CATEGORIES[8];
        let curr9 = cat9.eq_ignore_ascii_case(&modal.current_category);
        let style9 = if curr9 {
            Style::default().fg(CATPPUCCIN_GREEN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_TEXT)
        };
        lines.push(Line::from(vec![
            Span::styled(format!(" [{}] ", 9), Style::default().fg(CATPPUCCIN_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<15}", cat9), style9),
        ]));

        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(" [1-9] ", Style::default().fg(CATPPUCCIN_LAVENDER).add_modifier(Modifier::BOLD)),
            Span::raw("Select Category   "),
            Span::styled(" [Esc/q] ", Style::default().fg(CATPPUCCIN_RED).add_modifier(Modifier::BOLD)),
            Span::raw("Cancel"),
        ]));

        let p = Paragraph::new(lines);
        f.render_widget(p, inner);
    }
}
