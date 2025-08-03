use std::panic::{self};

pub fn add_panic_hook<F: Fn() -> std::io::Result<()> + Send + Sync>(cleanup: &'static F) {
    panic::update_hook(|old_hook, info| {
        cleanup().unwrap();

        old_hook(info);
    });
}
