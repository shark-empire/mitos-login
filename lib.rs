//! mitos-login — the MITOS login manager.
//!
//! Process model (see docs/architecture.md):
//!
//! * `mitos-login daemon`  — root, one per machine. Owns authentication,
//!   session launching, lockout policy, the audit log and the input devices.
//! * `mitos-login greeter` — spawned by the daemon as an unprivileged user
//!   with no groups, `no_new_privs`, and only two inherited descriptors (the
//!   IPC link and the framebuffer). It renders the Liquid Glass UI.
//! * `mitos-loginctl`      — admin CLI that talks to the daemon's control socket.
//!
//! All `unsafe` code in the crate lives in [`sys`].
#![deny(unsafe_code)]

pub mod accessibility;
pub mod authentication;
pub mod config;
pub mod daemon;
pub mod displays;
pub mod errors;
pub mod greeter;
pub mod greeter_ui;
pub mod ipc;
pub mod localization;
pub mod power;
pub mod seats;
pub mod security;
pub mod sessions;
#[allow(unsafe_code)]
pub mod sys;
pub mod theme;
pub mod users;

use std::path::Path;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONFIG_DIR: &str = "/etc/mitos-login";

/// Extracts `--config DIR` from `args`, falling back to [`DEFAULT_CONFIG_DIR`].
pub fn config_dir_from_args(args: &[String]) -> String {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--config" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
        i += 1;
    }
    DEFAULT_CONFIG_DIR.to_string()
}

/// Entry point shared by the `mitos-login` binary. Returns the exit code.
pub fn run(args: Vec<String>) -> i32 {
    let mut it = args.into_iter().skip(1);
    let cmd = it.next().unwrap_or_else(|| "daemon".to_string());
    let rest: Vec<String> = it.collect();
    match cmd.as_str() {
        "daemon" => daemon::run(&rest),
        "greeter" => greeter::run(&rest),
        "preview" => greeter::preview(&rest),
        "check-config" => check_config(&rest),
        "version" | "--version" | "-V" => {
            println!("mitos-login {}", VERSION);
            0
        }
        "help" | "--help" | "-h" => {
            print_help();
            0
        }
        other => {
            eprintln!("mitos-login: unknown command '{}'", other);
            print_help();
            2
        }
    }
}

fn print_help() {
    println!("mitos-login {}", VERSION);
    println!("usage: mitos-login <command> [--config DIR]");
    println!("  daemon         run the login daemon (default)");
    println!("  greeter        internal: run the graphical greeter (spawned by the daemon)");
    println!("  preview OUT.ppm [WxH] [scene]   render one greeter frame headlessly");
    println!("  check-config   validate the configuration and exit");
    println!("  version        print the version");
}

fn check_config(args: &[String]) -> i32 {
    let dir = config_dir_from_args(args);
    match config::Config::load(Path::new(&dir)) {
        Ok(_) => {
            println!("configuration in {} is valid", dir);
            0
        }
        Err(e) => {
            eprintln!("configuration error: {}", e);
            1
        }
    }
}
