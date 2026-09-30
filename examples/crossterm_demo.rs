use anyhow::Result;
use crossterm::ExecutableCommand; // 立即模式下的函数和宏接口
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal;
use std::io::{Write, stdout};
use std::time::Duration;

struct RawGuard;
impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

struct AltScreenGuard;
impl Drop for AltScreenGuard {
    fn drop(&mut self) {
        let _ = stdout().execute(terminal::LeaveAlternateScreen);
    }
}

fn is_quit_key(key_event: KeyEvent) -> bool {
    if key_event.kind == KeyEventKind::Release {
        return false;
    }

    match key_event.code {
        KeyCode::Char(c) if c.to_ascii_lowercase() == 'q' => true,
        KeyCode::Char(c)
            if c.to_ascii_lowercase() == 'c'
                && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            true
        }
        _ => false,
    }
}
fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    let _raw_guard = RawGuard;
    stdout.execute(terminal::EnterAlternateScreen)?;
    let _alt_screen_guard = AltScreenGuard;

    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        Print("this is alternate screen!\r\n")
    )?;
    stdout.flush()?;

    let mut counter = 0;
    let progress_frames = vec!['/', '-', '\\', '-'];

    let mut log_counter = 3;

    loop {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    if is_quit_key(key_event) {
                        break;
                    }

                    queue!(
                        stdout,
                        cursor::MoveTo(0, log_counter),
                        Print(format!("{:?}", key_event))
                    )?;
                    log_counter += 1;

                    if log_counter >= terminal::size()?.1 {
                        log_counter = 3;
                        queue!(
                            stdout,
                            cursor::MoveTo(0, log_counter),
                            terminal::Clear(terminal::ClearType::FromCursorDown),
                        )?;
                    }

                    stdout.flush()?;
                }
                _ => {}
            }
        } else {
            let frame = progress_frames[counter % progress_frames.len()];
            counter += 1;
            queue!(stdout, cursor::MoveTo(30, 0), Print(frame))?;
            stdout.flush()?;
        }
    }

    Ok(())
}
