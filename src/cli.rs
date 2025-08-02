use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    pub file_name: Option<String>,
}
