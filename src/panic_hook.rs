use anyhow::Result;
use std::panic::{self};

pub fn add_panic_hook<F: Fn() -> Result<()> + Send + Sync>(cleanup: &'static F) {
    panic::update_hook(|old_hook, info| {
        cleanup().expect("Cleanup failed");

        old_hook(info);
    });
}
