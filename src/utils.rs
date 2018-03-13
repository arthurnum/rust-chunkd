use std::time::Duration;
use std::thread;

pub fn sleep_nop(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
