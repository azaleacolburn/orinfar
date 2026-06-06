use crate::{mode::Mode, status_bar::StatusBar};
use anyhow::Result;

pub fn command(status_bar: &mut StatusBar, mode: &mut Mode) -> Result<()> {
    let full_command: String = status_bar[1..].iter().collect();

    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(full_command)
        .output()?;

    status_bar.clear();
    *mode = Mode::Normal;

    // TODO
    // In the future we might want a more sophisticated way of managing the output of commands

    Ok(())
}
