use std::panic::{self, take_hook};

pub fn set_cleanup_hook<T: Fn() -> std::io::Result<()> + Send + Sync>(cleanup: &'static T) {
    panic::update_hook(|old_hook, info| {
        cleanup().unwrap();
        old_hook(info);
    });
}
