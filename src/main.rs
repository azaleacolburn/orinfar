use std::io::stdout;

use crossterm::{
    cursor::{
        self, position, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, RestorePosition,
        SavePosition,
    },
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, ScrollDown, ScrollUp, SetSize,
    },
    ExecutableCommand,
};

#[derive(Clone, Debug)]
enum Mode {
    Normal,
    Insert,
    _Visual,
}

fn main() -> std::io::Result<()> {
    let (cols, rows) = size()?;
    let (cursor_cols, cursor_rows) = position()?;
    execute!(
        stdout(),
        SetSize(cols, rows),
        Clear(ClearType::All),
        ScrollUp(rows),
        SetForegroundColor(Color::Blue),
        // SetBackgroundColor(Color::Red),
        Print("Hello World"),
    )?;

    // Fill entire screen with spaces with the background color
    for row in 0..rows {
        execute!(stdout(), MoveTo(0, row), Print(" ".repeat(cols as usize)))?;
    }
    execute!(stdout(), MoveTo(0, rows))?;

    enable_raw_mode()?;

    let mut mode = Mode::Normal;

    let mut count: u16 = 1;

    loop {
        if let Event::Key(event) = read()? {
            match (event.code, mode.clone()) {
                (KeyCode::Char(c), Mode::Normal) if c == 'i' => mode = Mode::Insert,
                (KeyCode::Char(c), Mode::Normal) if c == 'm' => {
                    stdout().execute(Print(format!("{:?}", mode)))?;
                }
                (KeyCode::Char(c), Mode::Normal) if c == 'q' => break,
                (KeyCode::Char(c), Mode::Normal) if c.is_numeric() => {
                    let c = c.to_digit(10).unwrap() as u16;
                    if count == 1 {
                        count = 0;
                    }
                    count *= 10;
                    count += c;
                    stdout().execute(Print(format!("{:?}\n", count)))?;
                }

                (KeyCode::Esc, Mode::Insert) => {
                    mode = Mode::Normal;
                    count = 1;
                }
                (KeyCode::Backspace, Mode::Insert) => {
                    execute!(stdout(), MoveLeft(2), Print(" "), MoveRight(1))?;
                }
                (KeyCode::Char(c), Mode::Insert) => {
                    stdout().execute(Print(c))?;
                }

                (KeyCode::Left, _) => {
                    stdout().execute(MoveLeft(count))?;
                }
                (KeyCode::Right, _) => {
                    stdout().execute(MoveRight(count))?;
                }
                (KeyCode::Up, _) => {
                    stdout().execute(MoveUp(count))?;
                }
                (KeyCode::Down, _) => {
                    stdout().execute(MoveDown(count))?;
                }
                _ => continue,
            };
        }
    }

    execute!(stdout(), SetSize(cols, rows), ResetColor)?;
    disable_raw_mode()?;

    Ok(())
}
