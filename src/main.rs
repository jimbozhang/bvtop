mod app;
mod gpu;
mod ui;

use std::io::{self, Read as _, Write as _};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use gpu::GpuContext;

/// Probe whether the terminal renders box-drawing chars as 1 or 2 columns wide.
fn detect_char_width() -> u16 {
    let Ok(mut stdin) = std::fs::File::open("/dev/tty") else {
        return 1;
    };
    let mut stdout = io::stdout();

    // Enter raw mode temporarily to read cursor response
    let _ = enable_raw_mode();
    let _ = write!(stdout, "\x1b[6n");
    let _ = stdout.flush();

    // Skip any existing response
    let mut buf = [0u8; 32];
    let mut resp = Vec::new();
    while let Ok(n) = stdin.read(&mut buf) {
        if n == 0 {
            break;
        }
        resp.extend_from_slice(&buf[..n]);
        if resp.contains(&b'R') {
            break;
        }
    }

    // Write a box-drawing char and query cursor again
    let _ = write!(stdout, "╗\x1b[6n");
    let _ = stdout.flush();
    resp.clear();
    while let Ok(n) = stdin.read(&mut buf) {
        if n == 0 {
            break;
        }
        resp.extend_from_slice(&buf[..n]);
        if resp.contains(&b'R') {
            break;
        }
    }
    let _ = disable_raw_mode();

    // Clean up: move cursor back one position and clear
    let _ = write!(stdout, "\x1b[1D \x1b[1D");

    // Parse ESC[row;colR
    let s = String::from_utf8_lossy(&resp);
    if let Some(brace) = s.rfind('[') {
        if let Some(semicolon) = s[brace..].find(';') {
            let col_str = &s[brace + semicolon + 1..];
            let col_str = col_str.trim_end_matches('R');
            if let Ok(col) = col_str.parse::<u16>() {
                return if col >= 3 { 2 } else { 1 };
            }
        }
    }
    1 // default to single-width
}

fn main() -> Result<()> {
    let gpu_ctx = GpuContext::new()?;

    // Detect terminal char width before entering alternate screen
    let char_width = detect_char_width();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_secs(1);

    loop {
        // Check minimum terminal size
        let size = terminal.size()?;
        if size.width < ui::MIN_COLS || size.height < ui::MIN_ROWS {
            terminal.draw(|f| ui::draw_too_small(f, size.width, size.height))?;
        } else {
            terminal.draw(|f| ui::draw(f, &app, char_width))?;
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
                        || (key.code == KeyCode::Char('c')
                            && key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL))
                    {
                        break;
                    }
                }
                Event::Resize(_w, _h) => {}
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            match gpu_ctx.query() {
                Ok(info) => app.update(&info),
                Err(e) => {
                    cleanup(&mut terminal)?;
                    eprintln!("Failed to query GPU: {}", e);
                    break;
                }
            }

            last_tick = Instant::now();
        }
    }

    cleanup(&mut terminal)?;
    Ok(())
}

fn cleanup(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
