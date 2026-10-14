//! `mitos-loginctl`: talks to the daemon's control socket
//! (`/run/mitos-login/control.sock`, root-only via `SO_PEERCRED` — see
//! `src/ipc/permissions.rs`). Must be run as root.
use mitos_login::config::Config;
use mitos_login::ipc::client;
use mitos_login::ipc::messages::{ControlRequest, ControlResponse};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn socket_path(args: &[String]) -> PathBuf {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--config" && i + 1 < args.len() {
            if let Ok(cfg) = Config::load(Path::new(&args[i + 1])) {
                return cfg.control_socket_path;
            }
        }
        i += 1;
    }
    Config::load(Path::new(mitos_login::DEFAULT_CONFIG_DIR))
        .map(|c| c.control_socket_path)
        .unwrap_or_else(|_| PathBuf::from("/run/mitos-login/control.sock"))
}

fn print_help() {
    println!("mitos-loginctl {}", mitos_login::VERSION);
    println!("usage: mitos-loginctl <command> [--config DIR]");
    println!("  status            daemon version, backend, active session count");
    println!("  sessions          list active sessions (id, user, seat)");
    println!("  reload            ask the daemon to re-read its config files");
    println!("  audit-verify      check the audit log's tamper-evident hash chain");
    println!("  terminate <id>    send SIGTERM to a session's process");
    println!("  help              print this message");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    let rest = if args.len() > 2 { &args[2..] } else { &[] };

    let req = match cmd {
        "status" => ControlRequest::Status,
        "sessions" => ControlRequest::ListSessions,
        "reload" => ControlRequest::Reload,
        "audit-verify" => ControlRequest::VerifyAuditLog,
        "terminate" => match rest.first().and_then(|s| s.parse::<u32>().ok()) {
            Some(id) => ControlRequest::Terminate { session_id: id },
            None => {
                eprintln!("usage: mitos-loginctl terminate <session-id>");
                return ExitCode::from(2);
            }
        },
        "help" | "--help" | "-h" => {
            print_help();
            return ExitCode::SUCCESS;
        }
        "version" | "--version" | "-V" => {
            println!("mitos-loginctl {}", mitos_login::VERSION);
            return ExitCode::SUCCESS;
        }
        other => {
            eprintln!("mitos-loginctl: unknown command '{}'", other);
            print_help();
            return ExitCode::from(2);
        }
    };

    let path = socket_path(&args[1..]);
    match client::call(&path, &req) {
        Ok(resp) => {
            print_response(&resp);
            if matches!(resp, ControlResponse::Error(_) | ControlResponse::AuditTampered) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("mitos-loginctl: cannot reach the daemon at {}: {}", path.display(), e);
            eprintln!("(mitos-loginctl must be run as root, and the daemon must be running)");
            ExitCode::FAILURE
        }
    }
}

fn print_response(resp: &ControlResponse) {
    match resp {
        ControlResponse::Status { version, active_sessions, backend } => {
            println!("daemon version:   {}", version);
            println!("active sessions:  {}", active_sessions);
            println!("auth backend:     {}", backend);
        }
        ControlResponse::Sessions(list) => {
            if list.is_empty() {
                println!("no active sessions");
            } else {
                println!("{:<6} {:<16} {}", "ID", "USER", "SEAT");
                for (id, user, seat) in list {
                    println!("{:<6} {:<16} {}", id, user, seat);
                }
            }
        }
        ControlResponse::Ok => println!("ok"),
        ControlResponse::AuditOk => println!("audit log: chain intact"),
        ControlResponse::AuditTampered => println!("audit log: TAMPERED -- the hash chain is broken; see docs/security.md"),
        ControlResponse::Error(msg) => println!("error: {}", msg),
    }
}
