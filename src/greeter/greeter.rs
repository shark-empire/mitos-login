//! The greeter process's main loop: runs unprivileged, owns nothing but the
//! fds the daemon handed it (see docs/architecture.md's process model).
use super::animation::ease_out_quad;
use super::renderer::{backdrop_generate, Canvas};
use super::state::{Screen, State};
use crate::accessibility::Settings as AccessibilitySettings;
use crate::displays::Display;
use crate::ipc::messages::{GreeterEvent, GreeterRequest};
use crate::ipc::protocol::{decode_greeter_event, encode_greeter_request, read_frame, write_frame};
use crate::localization::locale;
use crate::seats::input::{InputEvent, Keymap};
use crate::sys::{self, PollFd, POLLIN};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

const FRAME_INTERVAL: Duration = Duration::from_millis(33); // ~30 fps
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

fn env_fd(name: &str) -> Option<i32> {
    std::env::var(name).ok()?.parse().ok()
}

pub fn run(_args: &[String]) -> i32 {
    let Some(fb_fd) = env_fd("MITOS_LOGIN_FB_FD") else {
        eprintln!("mitos-login greeter: missing MITOS_LOGIN_FB_FD (this binary is not meant to be run directly)");
        return 1;
    };
    let Some(ipc_fd) = env_fd("MITOS_LOGIN_IPC_FD") else {
        eprintln!("mitos-login greeter: missing MITOS_LOGIN_IPC_FD");
        return 1;
    };
    let input_base = env_fd("MITOS_LOGIN_INPUT_FD_BASE").unwrap_or(5);
    let input_count = env_fd("MITOS_LOGIN_INPUT_FD_COUNT").unwrap_or(0).max(0);

    let mut display = match Display::from_fd(fb_fd) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("mitos-login greeter: cannot use framebuffer: {}", e);
            return 1;
        }
    };
    let mut link = sys::adopt_fd_as_unix_stream(ipc_fd);
    let _ = link.set_nonblocking(true);

    let input_fds: Vec<std::fs::File> = (0..input_count).map(|i| sys::adopt_fd_as_file(input_base + i)).collect();

    let language = locale::detect_from_env();
    let mut state = State::new(language, AccessibilitySettings::default());
    let mut keymap = Keymap::new();

    let _ = write_frame(&mut link, &encode_greeter_request(&GreeterRequest::Ready));

    let start = Instant::now();
    let mut last_frame = Instant::now() - FRAME_INTERVAL;
    let mut last_heartbeat = Instant::now();
    let low_w = 160usize;
    let low_h = (low_w as f32 * display.height() as f32 / display.width() as f32).round().max(8.0) as usize;

    loop {
        if state.should_quit {
            return 0;
        }
        poll_once(&link, &input_fds, 10);

        drain_ipc(&mut link, &mut state);
        drain_input(&input_fds, &mut keymap, &mut state, &mut link);

        // state.idle_for() already tracks how long the login form has sat
        // untouched (see config/security.toml's greeter_idle_timeout_secs);
        // a dimmed idle view that uses it is not implemented yet -- see
        // docs/architecture.md's roadmap.

        if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
            let _ = write_frame(&mut link, &encode_greeter_request(&GreeterRequest::Heartbeat));
            last_heartbeat = Instant::now();
        }

        if last_frame.elapsed() >= FRAME_INTERVAL {
            last_frame = Instant::now();
            let t = start.elapsed().as_secs_f32();
            render_frame(&mut display, &state, t, low_w, low_h);
        } else {
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}

fn poll_once(link: &UnixStream, input_fds: &[std::fs::File], timeout_ms: i32) {
    let mut fds: Vec<PollFd> = Vec::with_capacity(1 + input_fds.len());
    fds.push(PollFd { fd: link.as_raw_fd(), events: POLLIN, revents: 0 });
    for f in input_fds {
        fds.push(PollFd { fd: f.as_raw_fd(), events: POLLIN, revents: 0 });
    }
    let _ = sys::poll(&mut fds, timeout_ms);
}

fn drain_ipc(link: &mut UnixStream, state: &mut State) {
    loop {
        match read_frame(link) {
            Ok(bytes) => match decode_greeter_event(&bytes) {
                Ok(event) => apply_event(state, event),
                Err(_) => continue,
            },
            Err(_) => break, // WouldBlock or the link closed; either way, nothing more to read right now
        }
    }
}

fn apply_event(state: &mut State, event: GreeterEvent) {
    match event {
        GreeterEvent::Show { users, sessions, last_user } => {
            state.users = users;
            state.sessions = sessions;
            if let Some(name) = last_user {
                if let Some(i) = state.users.iter().position(|u| u.name == name) {
                    state.selected_user = i;
                }
            }
        }
        GreeterEvent::AuthFeedback { outcome } => state.apply_outcome(outcome),
        GreeterEvent::Shake => state.shake_started = Some(Instant::now()),
        GreeterEvent::PowerCapabilities { can_suspend, can_reboot, can_poweroff } => {
            state.can_suspend = can_suspend;
            state.can_reboot = can_reboot;
            state.can_poweroff = can_poweroff;
        }
        GreeterEvent::PowerActionFailed { message } => {
            state.error_message = Some(message);
            state.screen = Screen::Login;
        }
        GreeterEvent::Farewell => state.should_quit = true,
    }
}

fn drain_input(input_fds: &[std::fs::File], keymap: &mut Keymap, state: &mut State, link: &mut UnixStream) {
    for f in input_fds {
        for raw in sys::evdev_read(f.as_raw_fd()) {
            if let Some(ev) = keymap.translate(raw) {
                dispatch(state, keymap, ev, link);
            }
        }
    }
}

fn dispatch(state: &mut State, keymap: &Keymap, ev: InputEvent, link: &mut UnixStream) {
    match super::input::handle(state, keymap.mods.caps_lock, ev) {
        super::input::Action::None => {}
        super::input::Action::Send(req) => {
            let _ = write_frame(link, &encode_greeter_request(&req));
        }
        super::input::Action::Quit => state.should_quit = true,
    }
}

fn render_frame(display: &mut Display, state: &State, t: f32, low_w: usize, low_h: usize) {
    let backdrop = backdrop_generate(low_w, low_h, t);
    let theme = crate::theme::Theme::new(
        display.height(),
        state.accessibility.high_contrast,
        state.accessibility.large_text,
        state.accessibility.reduced_motion,
    );
    let mut canvas = Canvas::new(display.width() as usize, display.height() as usize, backdrop);
    let materialize = ease_out_quad(state.materialized_at.elapsed().as_secs_f32() / 0.4);
    let _ = materialize; // reserved for a future fade/scale-in on first frame
    let now = chrono_like_now();
    super::screen::draw(&mut canvas, &theme, state, t, &now.0, &now.1);
    let _ = display.present(&canvas);
}

/// A tiny local clock formatter (`HH:MM`, `Weekday, D Month`) — zero
/// dependencies, so no `chrono`; good enough for a status display that is
/// never used for anything date-arithmetic-sensitive.
fn chrono_like_now() -> (String, String) {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let days_since_epoch = (secs / 86400) as i64;
    let time_of_day = secs % 86400;
    let (h, m) = (time_of_day / 3600, (time_of_day % 3600) / 60);
    let (y, mo, d) = civil_from_days(days_since_epoch);
    let weekday = ["Thursday", "Friday", "Saturday", "Sunday", "Monday", "Tuesday", "Wednesday"][(days_since_epoch.rem_euclid(7)) as usize];
    let month = [
        "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December",
    ][(mo - 1) as usize];
    let _ = y; // the greeter's date line intentionally omits the year (screen space, matches the design in docs/greeter.md)
    (format!("{:02}:{:02}", h, m), format!("{}, {} {}", weekday, d, month))
}

/// Howard Hinnant's `civil_from_days` algorithm (public domain), the same
/// one `mitos-utils`' own `date` applet independently verified against
/// Python's `datetime` across 200k+ days.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as i64;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as i64;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19618), (2023, 9, 18));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }
}
