//! The root daemon: owns the seat, spawns the greeter, authenticates, and
//! launches sessions. See docs/architecture.md for the full picture.
pub mod daemon;
pub mod event_loop;
pub mod lifecycle;

use crate::config_dir_from_args;

/// Entry point for `mitos-login daemon`.
pub fn run(args: &[String]) -> i32 {
    let dir = config_dir_from_args(args);
    match daemon::Daemon::start(std::path::Path::new(&dir)) {
        Ok(mut d) => d.run_forever(),
        Err(e) => {
            eprintln!("mitos-login: failed to start: {}", e);
            1
        }
    }
}
