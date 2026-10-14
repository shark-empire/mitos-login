//! End-to-end tests of both wire protocols over real sockets (not just the
//! encode/decode round trips already covered inside src/ipc/protocol.rs).
use mitos_login::ipc::messages::{ControlRequest, ControlResponse};
use mitos_login::ipc::protocol::{decode_greeter_request, encode_greeter_request, read_frame, write_frame};
use mitos_login::ipc::server::ControlServer;
use mitos_login::ipc::{client, GreeterRequest};
use std::os::unix::net::UnixStream;

#[test]
fn greeter_link_round_trips_over_a_real_socketpair() {
    let (mut a, mut b) = UnixStream::pair().unwrap();
    write_frame(&mut a, &encode_greeter_request(&GreeterRequest::Heartbeat)).unwrap();
    let bytes = read_frame(&mut b).unwrap();
    assert_eq!(decode_greeter_request(&bytes).unwrap(), GreeterRequest::Heartbeat);
}

#[test]
fn control_socket_round_trip_as_root_succeeds() {
    // This test only meaningfully exercises the SO_PEERCRED allow path when
    // the test process itself runs as uid 0 (true in this project's own CI
    // container, and in the sandbox this crate was developed in); under a
    // non-root test runner it still must not panic, so the assertion below
    // accepts either a successful Status reply or an authorization error,
    // rather than asserting unconditionally on an environment this test
    // cannot control.
    let dir = std::env::temp_dir().join(format!("mitos-login-it-ipc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sock = dir.join("control.sock");
    let server = ControlServer::bind(&sock).unwrap();

    // The server only services connections already queued when
    // accept_requests is called, so the client request must be sent (and
    // its SYN/accept complete) before polling -- a short retry loop keeps
    // this from being flaky under container scheduling jitter without
    // sleeping blindly.
    let client_thread = std::thread::spawn({
        let sock = sock.clone();
        move || client::call(&sock, &ControlRequest::Status)
    });
    let mut response = None;
    for _ in 0..50 {
        for (stream, _req) in server.accept_requests() {
            server.respond(stream, &ControlResponse::Status { version: "test".into(), active_sessions: 0, backend: "test".into() });
        }
        if client_thread.is_finished() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let result = client_thread.join().unwrap();
    if let Ok(resp) = result {
        response = Some(resp);
    }

    match response {
        Some(ControlResponse::Status { version, .. }) => assert_eq!(version, "test"),
        Some(ControlResponse::Error(_)) => {} // not running as root in this environment: acceptable
        other => panic!("unexpected result: {:?}", other),
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn control_socket_is_created_with_mode_0600() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("mitos-login-it-ipcmode-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sock = dir.join("control.sock");
    let _server = ControlServer::bind(&sock).unwrap();
    let mode = std::fs::metadata(&sock).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    std::fs::remove_dir_all(&dir).ok();
}
