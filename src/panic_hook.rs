use anyhow::Result;
use std::panic::{self};

pub fn add_panic_hook<F: Fn() -> Result<()> + Send + Sync>(cleanup: &'static F) {
    panic::set_hook(Box::new(|info| {
        cleanup().unwrap_or_else(|_| panic!("Cleanup failed: {info}"));

        println!("{info}");
    }));
}
