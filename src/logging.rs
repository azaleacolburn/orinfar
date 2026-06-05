use std::{fs::OpenOptions, io::Write, path::PathBuf};

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
