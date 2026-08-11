use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub const MIN_COLS: u16 = 60;
pub const MIN_ROWS: u16 = 20;

// ── Big digits (5-row block font) ──────────────────────────────

const DIGITS: [[&str; 5]; 10] = [
    [" █████╗ ", "██╔███╗ ", "██║╚██║ ", "██║ ██║ ", "╚████╔╝ "],
    [" ███╗  ", "████║  ", "╚██║   ", " ██║   ", "██████╗"],
    ["██████╗ ", "╚════██║", " █████╔╝", "██╔═══╝ ", "███████╗"],
    ["██████╗ ", "╚════██║", " █████╔╝", " ╚═══██║", "██████╔╝"],
    ["██╗  ██╗", "██║  ██║", "███████║", "╚════██║", "     ██║"],
    ["███████╗", "██╔════╝", "███████╗", "╚════██║", "███████║"],
    [" ██████╗", "██╔════╝", "███████╗", "██╔══██║", "╚█████╔╝"],
    ["███████╗", "╚════██║", "   ██╔╝ ", "  ██╔╝  ", "  ██║   "],
    [" █████╗ ", "██╔══██╗", "╚█████╔╝", "██╔══██╗", "╚█████╔╝"],
    [" █████╗ ", "██╔══██╗", "╚██████╗", " ╚═══██║", " █████╔╝"],
];

const DOT: [&str; 5] = ["        ", "        ", "        ", "        ", "  ██╗   "];

const SLASH: [&str; 5] = ["    ██╗ ", "   ██╔╝ ", "  ██╔╝  ", " ██╔╝   ", "██╔╝    "];

const SPACE: [&str; 5] = ["        ", "        ", "        ", "        ", "        "];

fn glyph_for(ch: char, row: usize) -> &'static str {
    match ch {
        '0'..='9' => DIGITS[(ch as u8 - b'0') as usize][row],
        '.' => DOT[row],
        '/' => SLASH[row],
        _ => SPACE[row],
    }
}

/// Style: █ = White (body), box-drawing = Gray (outline), space = default
fn styled_line(text: &str, row: usize) -> Line<'static> {
    let mut spans = Vec::new();
    for ch in text.chars() {
        let glyph = glyph_for(ch, row);
        for c in glyph.chars() {
            let color = match c {
                '\u{2588}' => Color::White, // █ full block → bright body
                '\u{2550}'..='\u{256F}' | '\u{2570}'..='\u{257F}' => Color::DarkGray, // box-drawing → outline
                _ => Color::DarkGray, // space → invisible but set color anyway
            };
            let s: &str = Box::leak(c.to_string().into_boxed_str());
            spans.push(Span::styled(s, Style::default().fg(color)));
        }
    }
    Line::from(spans)
}

fn render_big_text_lines(text: &str) -> Vec<Line<'static>> {
    (0..5).map(|row| styled_line(text, row)).collect()
}

// ── UI ─────────────────────────────────────────────────────────

fn render_quadrant(f: &mut Frame, area: Rect, label: &str, value: &str) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(5)])
        .split(outer[1]);

    // Label: cyan, centered
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            label,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        inner[0],
    );

    // Big digits: styled (white body + gray outline), centered
    let lines = render_big_text_lines(value);
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner[1]);
}

pub fn draw(f: &mut Frame, app: &App, _char_width: u16) {
    let size = f.area();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(size);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_quadrant(
        f,
        top_cols[0],
        "GPU UTILIZATION (%)",
        &app.gpu_util.to_string(),
    );
    render_quadrant(f, top_cols[1], "VRAM USAGE (GB)", &app.mem_val_display());
    render_quadrant(
        f,
        bottom_cols[0],
        "TEMPERATURE (°C)",
        &app.temperature.to_string(),
    );
    render_quadrant(f, bottom_cols[1], "POWER (W)", &app.power_val_display());
}

pub fn draw_too_small(f: &mut Frame, cols: u16, rows: u16) {
    let msg = format!(
        "Window too small ({}x{}). Minimum: {}x{}",
        cols, rows, MIN_COLS, MIN_ROWS
    );
    let area = f.area();
    let line = Line::from(Span::styled(
        msg,
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    ));
    let para = Paragraph::new(line).alignment(Alignment::Center);
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);
    f.render_widget(para, outer[1]);
}
