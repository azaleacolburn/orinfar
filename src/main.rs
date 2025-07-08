use std::io::stdout;

use crossterm::{
    cursor::{self, position, MoveDown, MoveLeft, MoveRight, MoveUp},
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, ScrollUp, SetSize},
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
        ScrollUp(cols),
        SetForegroundColor(Color::Blue),
        SetBackgroundColor(Color::Red),
        Print("Hello World"),
    )?;

    enable_raw_mode()?;

    let mut mode = Mode::Normal;

    let mut count = 1;

    loop {
        if let Event::Key(event) = read()? {
            match (event.code, mode.clone()) {
                (KeyCode::Char(c), Mode::Normal) if c == 'i' => mode = Mode::Insert,
                (KeyCode::Char(c), Mode::Normal) if c == 'm' => {
                    stdout().execute(Print(format!("{:?}", mode)))?;
                    ()
                }
                (KeyCode::Char(c), Mode::Normal) if c == 'q' => break,
                (KeyCode::Esc, Mode::Insert) => mode = Mode::Normal,
                (KeyCode::Char(c), Mode::Insert) => {
                    stdout().execute(Print(c))?;
                    ()
                }

                (KeyCode::Left, _) => {
                    stdout().execute(MoveLeft(count))?;
                    ()
                }
                (KeyCode::Right, _) => {
                    stdout().execute(MoveRight(count))?;
                    ()
                }
                (KeyCode::Up, _) => {
                    stdout().execute(MoveUp(count))?;
                    ()
                }
                (KeyCode::Down, _) => {
                    stdout().execute(MoveDown(count))?;
                    ()
                }
                _ => continue,
            };
        }
    }

    execute!(stdout(), SetSize(cols, rows), ResetColor)?;
    disable_raw_mode()?;

    Ok(())
}
