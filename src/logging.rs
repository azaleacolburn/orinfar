use anyhow::Result;
use std::{fs::OpenOptions, io::Write, path::PathBuf};

pub struct OrinfarData {
    /// Whether they have opened Orinfar before on this machine
    pub has_opened: bool,
}

/// # Returns
/// - On a success, a `OrinfarData` structure containing data about the user's Orinfar use
pub fn setup_logging_and_data() -> Result<OrinfarData> {
    // This could fail if the dir already exists, so we don't care if this fails
    if let Err(err) = std::fs::create_dir(log_dir())
        && err.to_string() != "File exists (os error 17)"
    {
        return Err(err.into());
    }
    std::fs::File::create(log_file())?;
    let data_path = data_file();
    if !data_path.exists() {
        std::fs::File::create(&data_path)?;
    }

    let data = std::fs::read_to_string(&data_path)?;

    let mut has_opened = false;
    data.lines().for_each(|l| {
        let Some((k, v)) = l.split_once(':') else {
            panic!("Invalid Data File");
        };

        has_opened |= ("has_opened", "true") == (k.trim(), v.trim());
    });

    Ok(OrinfarData { has_opened })
}

pub fn log_dir() -> PathBuf {
    let base = xdg::BaseDirectories::with_prefix("orinfar");
    base.get_state_home().expect("Could not find home")
}

pub fn log_file() -> PathBuf {
    log_dir().join("log")
}

pub fn data_file() -> PathBuf {
    log_dir().join("data")
}

pub fn log(contents: &impl ToString) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(log_file())
        .expect("unable to open file");

    // append data to the file
    file.write_all(format!("{}\n", contents.to_string()).as_bytes())
        .expect("unable to append data");
}

pub fn write_data(key: &impl ToString, value: &impl ToString) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(data_file())
        .expect("unable to open file");

    // append data to the file
    file.write_all(format!("{}:{}\n", key.to_string(), value.to_string()).as_bytes())
        .expect("unable to append data");
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if *DEBUG.get().unwrap() {
            $crate::logging::log(&format!($($arg)*))
        }
    };
}
