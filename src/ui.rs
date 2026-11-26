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

// ── Palette ────────────────────────────────────────────────────

/// Background tint for 1%..=89% GPU utilization.
const UTIL_BG_BUSY: Color = Color::Rgb(0xB8, 0xDC, 0xFE);
/// Background tint for >= 90% GPU utilization.
const UTIL_BG_HIGH: Color = Color::Rgb(0xFF, 0xA9, 0x03);

/// Colors used to render the dashboard at a given GPU utilization.
#[derive(Clone, Copy)]
struct Palette {
    bg: Color,
    label: Color,
    body: Color,
    outline: Color,
}

impl Palette {
    fn for_util(util: u32) -> Self {
        if util == 0 {
            // Terminal default background; original color scheme.
            Self {
                bg: Color::Reset,
                label: Color::Cyan,
                body: Color::White,
                outline: Color::DarkGray,
            }
        } else {
            // Light background tint: switch to black ink for legibility.
            Self {
                bg: if util < 90 {
                    UTIL_BG_BUSY
                } else {
                    UTIL_BG_HIGH
                },
                label: Color::Black,
                body: Color::Black,
                outline: Color::Black,
            }
        }
    }
}

/// Style: █ = body color, box-drawing/space = outline color
fn styled_line(text: &str, row: usize, palette: Palette) -> Line<'static> {
    let mut spans = Vec::new();
    for ch in text.chars() {
        let glyph = glyph_for(ch, row);
        for c in glyph.chars() {
            let color = match c {
                '\u{2588}' => palette.body, // █ full block → body
                _ => palette.outline,       // box-drawing / space → outline
            };
            spans.push(Span::styled(c.to_string(), Style::default().fg(color)));
        }
    }
    Line::from(spans)
}

fn render_big_text_lines(text: &str, palette: Palette) -> Vec<Line<'static>> {
    (0..5).map(|row| styled_line(text, row, palette)).collect()
}

// ── UI ─────────────────────────────────────────────────────────

fn render_quadrant(f: &mut Frame, area: Rect, label: &str, value: &str, palette: Palette) {
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

    // Label: palette label color, centered
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            label,
            Style::default()
                .fg(palette.label)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        inner[0],
    );

    // Big digits: styled (body + outline), centered
    let lines = render_big_text_lines(value, palette);
    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner[1]);
}

pub fn draw(f: &mut Frame, app: &App, _char_width: u16) {
    let size = f.area();
    let palette = Palette::for_util(app.gpu_util);

    // Tint the whole background; Color::Reset restores the terminal default.
    f.buffer_mut()
        .set_style(size, Style::default().bg(palette.bg));

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
        palette,
    );
    render_quadrant(
        f,
        top_cols[1],
        "VRAM USAGE (GB)",
        &app.mem_val_display(),
        palette,
    );
    render_quadrant(
        f,
        bottom_cols[0],
        "TEMPERATURE (°C)",
        &app.temperature.to_string(),
        palette,
    );
    render_quadrant(
        f,
        bottom_cols[1],
        "POWER (W)",
        &app.power_val_display(),
        palette,
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use ratatui::{backend::TestBackend, Terminal};

    fn bg_at(util: u32, x: u16, y: u16) -> Color {
        let mut app = App::new();
        app.gpu_util = util;
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| draw(f, &app, 1)).unwrap();
        terminal.backend().buffer()[(x, y)].bg
    }

    #[test]
    fn background_reflects_utilization() {
        // 0%: terminal default background.
        assert_eq!(bg_at(0, 0, 0), Color::Reset);
        assert_eq!(bg_at(0, 20, 6), Color::Reset);
        // 1%..=89%: light blue.
        assert_eq!(bg_at(1, 0, 0), UTIL_BG_BUSY);
        assert_eq!(bg_at(89, 20, 6), UTIL_BG_BUSY);
        // >= 90%: amber, including cells covered by widgets.
        assert_eq!(bg_at(90, 0, 0), UTIL_BG_HIGH);
        assert_eq!(bg_at(100, 20, 6), UTIL_BG_HIGH);
        assert_eq!(bg_at(100, 79, 23), UTIL_BG_HIGH);
    }
}
