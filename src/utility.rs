use std::{fs::OpenOptions, io::Write};

pub fn log(contents: impl ToString) {
    let mut file = OpenOptions::new()
        .append(true)
        .open("log.txt")
        .expect("Unable to open file");

    // Append data to the file
    file.write_all(format!("{}\n", contents.to_string()).as_bytes())
        .expect("Unable to append data");
}

macro_rules! unwrap_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

macro_rules! unwrap_or_break {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => break,
        }
    };
}
