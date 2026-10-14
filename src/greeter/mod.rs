//! The greeter: an unprivileged process that renders the Liquid Glass login
//! UI and relays what the person does to the daemon over the inherited IPC
//! link. See [`greeter::run`] for the real entry point and
//! `docs/greeter.md` for the rendering design.
pub mod animation;
pub mod font_data;
pub mod greeter;
pub mod input;
pub mod renderer;
pub mod screen;
pub mod state;

use crate::accessibility::Settings as AccessibilitySettings;
use crate::ipc::messages::{SessionSummary, UserSummary};
use crate::localization::Language;

/// Real entry point for `mitos-login greeter` (spawned by the daemon; see
/// [`crate::daemon::lifecycle::spawn_greeter`] for the fds/env it provides).
pub fn run(args: &[String]) -> i32 {
    greeter::run(args)
}

/// Renders exactly one frame to a `.ppm` file and exits — no daemon, no
/// root, no real hardware. Useful for design review and for
/// `scripts/render-preview.sh`. Usage: `mitos-login preview OUT.ppm
/// [WxH] [scene]`, `scene` one of `login` (default), `locked-out`,
/// `power-menu`, `error`.
pub fn preview(args: &[String]) -> i32 {
    let Some(out_path) = args.first() else {
        eprintln!("usage: mitos-login preview OUT.ppm [WxH] [scene]");
        return 2;
    };
    let (w, h) = args
        .get(1)
        .and_then(|s| s.split_once('x'))
        .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
        .unwrap_or((1280u32, 720u32));
    let scene = args.get(2).map(|s| s.as_str()).unwrap_or("login");

    let mut state = state::State::new(Language::EnUs, AccessibilitySettings::default());
    state.users = vec![
        UserSummary { name: "alex".into(), display_name: "Alex Rivera".into(), uid: 1000, has_avatar: false, locked: false },
        UserSummary { name: "mia".into(), display_name: "Mia Chen".into(), uid: 1001, has_avatar: false, locked: false },
        UserSummary { name: "sam".into(), display_name: "Sam Okafor".into(), uid: 1002, has_avatar: false, locked: false },
    ];
    state.selected_user = 0;
    state.sessions = vec![SessionSummary { id: "mitos".into(), name: "MITOS Desktop".into() }];
    state.can_suspend = true;
    state.can_reboot = true;
    state.can_poweroff = true;
    for c in "hunter2".chars() {
        state.push_password_char(c, 512);
    }
    match scene {
        "locked-out" => {
            state.locked_out_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(42));
            state.error_message = Some(state.translator.get_with("too_many_attempts", &[("seconds", "42")]));
        }
        "error" => {
            state.error_message = Some(state.translator.get("wrong_password").to_string());
            state.caps_lock = true;
        }
        "power-menu" => {
            state.screen = state::Screen::PowerMenu;
        }
        _ => {}
    }

    let theme = crate::theme::Theme::new(h, false, false, false);
    let low_w = 160usize;
    let low_h = (low_w as f32 * h as f32 / w as f32).round().max(8.0) as usize;
    let backdrop = renderer::backdrop_generate(low_w, low_h, 3.2);
    let mut canvas = renderer::Canvas::new(w as usize, h as usize, backdrop);
    screen::draw(&mut canvas, &theme, &state, 3.2, "09:41", "Thursday, 1 October");

    match write_ppm(out_path, &canvas) {
        Ok(()) => {
            println!("wrote {}", out_path);
            0
        }
        Err(e) => {
            eprintln!("mitos-login preview: {}", e);
            1
        }
    }
}

fn write_ppm(path: &str, canvas: &renderer::Canvas) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(f, "P6\n{} {}\n255\n", canvas.w, canvas.h)?;
    let mut row = vec![0u8; canvas.w * 3];
    for y in 0..canvas.h {
        for x in 0..canvas.w {
            let c = canvas.pixel(x, y);
            row[x * 3] = (c.r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            row[x * 3 + 1] = (c.g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            row[x * 3 + 2] = (c.b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
        f.write_all(&row)?;
    }
    Ok(())
}
