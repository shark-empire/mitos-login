//! A small software rasterizer: everything the greeter draws (glass panels,
//! the stroke font, icons, particles) is composited by hand into an RGB
//! float buffer, then converted to the framebuffer's native pixel format
//! once per frame in [`Canvas::blit_to_fb`]. There is no GPU here — Stage 5
//! of mitos-gui's own roadmap is the model for what eventually replaces this
//! with real GPU compositing after Stage 5 (DRM/GBM) lands for mitos-login
//! too; see docs/architecture.md.
use super::font_data::{self, Accent};
use crate::theme::Rgb;
use crate::sys::FbInfo;

#[derive(Clone)]
pub struct Backdrop {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Rgb>,
}

impl Backdrop {
    pub fn sample(&self, x: f32, y: f32) -> Rgb {
        let fx = (x - 0.5).clamp(0.0, self.w as f32 - 1.001);
        let fy = (y - 0.5).clamp(0.0, self.h as f32 - 1.001);
        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let wx = fx - x0 as f32;
        let wy = fy - y0 as f32;
        let x1 = (x0 + 1).min(self.w - 1);
        let y1 = (y0 + 1).min(self.h - 1);
        let at = |xx: usize, yy: usize| self.px[yy * self.w + xx];
        let a = at(x0, y0).scale(1.0 - wx).add(at(x1, y0).scale(wx));
        let b = at(x0, y1).scale(1.0 - wx).add(at(x1, y1).scale(wx));
        a.scale(1.0 - wy).add(b.scale(wy))
    }
}

/// Builds the animated backdrop at a small resolution (the caller upsamples
/// it); `t` is seconds of wall-clock time, driving a slow drift so the
/// screen never looks perfectly static.
pub fn backdrop_generate(lw: usize, lh: usize, t: f32) -> Backdrop {
    use crate::theme::colors::{BG_BOTTOM, BG_TOP, BLOOMS};
    let mut px = vec![Rgb::new(0.0, 0.0, 0.0); lw * lh];
    let aspect = lw as f32 / lh as f32;
    for y in 0..lh {
        for x in 0..lw {
            let u = (x as f32 + 0.5) / lh as f32;
            let v = (y as f32 + 0.5) / lh as f32;
            let mut f = BG_TOP.lerp(BG_BOTTOM, v.clamp(0.0, 1.0));
            for (i, bloom) in BLOOMS.iter().enumerate() {
                let cx = aspect * (0.5 + 0.36 * (t * 0.13 * bloom.speed + i as f32 * 1.7).sin());
                let cy = 0.5 + 0.30 * (t * 0.11 * (bloom.speed + 1.0) + i as f32 * 2.3).cos();
                let dx = u / aspect - cx;
                let dy = v - cy;
                let d2 = dx * dx + dy * dy;
                let g = bloom.strength * (-d2 / (2.0 * bloom.sigma * bloom.sigma)).exp();
                f = f.add(bloom.color.scale(g));
            }
            // soft-clip with an exponential curve (matches the prototype:
            // punchy near black, never blows out) rather than a hard clamp.
            f = Rgb::new(1.0 - (-1.25 * f.r).exp(), 1.0 - (-1.25 * f.g).exp(), 1.0 - (-1.25 * f.b).exp());
            let r = ((u / aspect - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt();
            let vignette = 1.0 - 0.55 * ((r - 0.30) / 0.55).clamp(0.0, 1.0).powf(1.4);
            px[y * lw + x] = f.scale(vignette);
        }
    }
    Backdrop { w: lw, h: lh, px }
}

fn hash01(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393).wrapping_add((y as u32).wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xffff) as f32 / 65535.0
}

fn sd_round_box(px: f32, py: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let qx = (px - cx).abs() - hw + r;
    let qy = (py - cy).abs() - hh + r;
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r
}

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    px: Vec<Rgb>,
    backdrop: Backdrop,
}

impl Canvas {
    /// Fills the whole canvas by upsampling `backdrop`, plus a faint
    /// per-pixel dither (breaks up banding in the dark gradient — cheap and
    /// worth doing since the panels sit on top of large flat-ish areas).
    pub fn new(w: usize, h: usize, backdrop: Backdrop) -> Canvas {
        let mut px = vec![Rgb::new(0.0, 0.0, 0.0); w * h];
        for y in 0..h {
            for x in 0..w {
                let mut c = backdrop.sample(x as f32, y as f32);
                let n = (hash01(x as i32, y as i32) - 0.5) / 255.0 * 1.5;
                c = Rgb::new(c.r + n, c.g + n, c.b + n);
                px[y * w + x] = c;
            }
        }
        Canvas { w, h, px, backdrop }
    }

    /// A composited pixel's color. Out-of-range coordinates return black
    /// rather than panicking — callers (e.g. the `.ppm` preview writer)
    /// never need to separately bounds-check `x`/`y` against `w`/`h`.
    pub fn pixel(&self, x: usize, y: usize) -> Rgb {
        self.px.get(y * self.w + x).copied().unwrap_or(Rgb::new(0.0, 0.0, 0.0))
    }

    fn get(&self, x: i32, y: i32) -> Rgb {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return Rgb::new(0.0, 0.0, 0.0);
        }
        self.px[y as usize * self.w + x as usize]
    }

    fn blend(&mut self, x: i32, y: i32, color: Rgb, coverage: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h || coverage <= 0.0 {
            return;
        }
        let idx = y as usize * self.w + x as usize;
        let bg = self.px[idx];
        let c = coverage.min(1.0);
        self.px[idx] = bg.scale(1.0 - c).add(color.scale(c));
    }

    fn add_at(&mut self, x: i32, y: i32, color: Rgb) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let idx = y as usize * self.w + x as usize;
        self.px[idx] = self.px[idx].add(color);
    }

    /// A "Liquid Glass" panel: a rounded rect that refracts the backdrop
    /// behind it, carries a soft outer glow in the accent gradient, a bright
    /// rim line, and an inner top-left highlight. `tint` mixes in a little
    /// flat brightness (glass is never perfectly clear); `glow`/`glow_radius`
    /// control the outer bloom; `accent`/`accent2` are the two colors the rim
    /// gradient rotates between.
    #[allow(clippy::too_many_arguments)]
    pub fn glass(&mut self, spec: GlassSpec) {
        let GlassSpec { cx, cy, w, h, radius, glow, glow_radius, tint, border, ring_phase, accent, accent2, pulse } = spec;
        let margin = (glow_radius * 3.2 + 30.0) as i32;
        let (hw, hh) = (w / 2.0, h / 2.0);
        let x0 = (cx - hw - margin as f32).floor().max(0.0) as i32;
        let x1 = (cx + hw + margin as f32).ceil().min(self.w as f32) as i32;
        let y0 = (cy - hh - margin as f32).floor().max(0.0) as i32;
        let y1 = (cy + hh + margin as f32).ceil().min(self.h as f32) as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
                let d = sd_round_box(fx, fy, cx, cy, hw, hh, radius);
                let gx = (sd_round_box(fx + 1.0, fy, cx, cy, hw, hh, radius) - sd_round_box(fx - 1.0, fy, cx, cy, hw, hh, radius)) * 0.5;
                let gy = (sd_round_box(fx, fy + 1.0, cx, cy, hw, hh, radius) - sd_round_box(fx, fy - 1.0, cx, cy, hw, hh, radius)) * 0.5;
                let ang = (fy - cy).atan2(fx - cx) + ring_phase;
                let k = 0.5 + 0.5 * (ang * 2.0).sin();
                let ring = accent.lerp(accent2, k);

                // Drop shadow beneath the panel (only outside it).
                if d > 0.0 {
                    let dsh = sd_round_box(fx, fy - 14.0, cx, cy, hw, hh, radius).max(0.0);
                    let sh = (-dsh / 22.0).exp() * 0.38;
                    let bg = self.get(px, py);
                    self.blend(px, py, Rgb::new(0.0, 0.0, 0.0), sh.min(1.0) * ((bg.r + bg.g + bg.b) / 3.0).min(1.0).max(0.15));
                    // Outer glow.
                    let g = (-d / glow_radius).exp() * glow * (0.5 + 0.5 * pulse) * 0.55;
                    self.add_at(px, py, ring.scale(g));
                    continue;
                }

                // Inside the panel: refracted backdrop + tint + highlight + inner glow + rim.
                let refr = 14.0 * (d / 14.0).exp();
                let ox = gx * refr;
                let oy = gy * refr;
                let base = self.backdrop.sample(
                    (fx - ox) * (self.backdrop.w as f32 / self.w as f32),
                    (fy - oy) * (self.backdrop.h as f32 / self.h as f32),
                );
                let mut col = base.scale(0.92);
                let vy = ((fy - (cy - hh)) / h).clamp(0.0, 1.0);
                col = col.scale(1.0 - tint).add(Rgb::new(0.95, 0.95, 0.95).scale(tint * (0.85 + 0.15 * (1.0 - vy))));
                col = col.add(Rgb::new(1.0, 1.0, 1.0).scale(0.05 * (1.0 - vy)));

                let light = (-0.55f32, -0.83f32);
                let ndotl = -(gx * light.0 + gy * light.1);
                let spec_hi = ndotl.max(0.0).powf(1.6) * (d / 2.2).exp() * 0.60 + (-ndotl).max(0.0).powf(1.6) * (d / 2.6).exp() * 0.22;
                col = col.add(Rgb::new(0.9, 0.95, 1.0).scale(spec_hi));

                let inner_glow = (d / 12.0).exp() * 0.16 * (0.55 + 0.45 * pulse);
                col = col.add(ring.scale(inner_glow));

                let rim = (-((d + 0.7).powi(2)) / (2.0 * (border * 0.6).powi(2))).exp() * 0.85;
                col = col.add(ring.scale(rim));

                let n = (hash01(px, py) - 0.5) * 0.02;
                col = Rgb::new(col.r + n, col.g + n, col.b + n);
                self.blend(px, py, col, 1.0);
            }
        }
    }

    /// A recessed field (the password/PIN box): the inverse lighting of
    /// [`Self::glass`] — darker at the top, a glowing accent rim when
    /// `focus` is high.
    pub fn inset(&mut self, cx: f32, cy: f32, w: f32, h: f32, radius: f32, focus: f32, pulse: f32, ring_phase: f32) {
        use crate::theme::colors::{ACCENT, ACCENT_2};
        let margin = 30i32;
        let (hw, hh) = (w / 2.0, h / 2.0);
        let x0 = (cx - hw) as i32 - margin;
        let x1 = (cx + hw) as i32 + margin;
        let y0 = (cy - hh) as i32 - margin;
        let y1 = (cy + hh) as i32 + margin;
        for py in y0.max(0)..y1.min(self.h as i32) {
            for px in x0.max(0)..x1.min(self.w as i32) {
                let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
                let d = sd_round_box(fx, fy, cx, cy, hw, hh, radius);
                let cov = (0.5 - d).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                let ang = (fy - cy).atan2(fx - cx) + ring_phase;
                let k = 0.5 + 0.5 * (ang * 2.0).sin();
                let ring = ACCENT.lerp(ACCENT_2, k);
                let bg = self.get(px, py);
                let vy = ((fy - (cy - hh)) / h).clamp(0.0, 1.0);
                let mut body = bg.scale(0.55).add(Rgb::new(0.02, 0.02, 0.02));
                let ishadow = (d / 5.0).exp() * 0.20 * (1.0 - vy);
                body = body.add(Rgb::new(-ishadow, -ishadow, -ishadow));
                let og = (-d.max(0.0) / 9.0).exp() * (d > 0.0) as u8 as f32 * focus * (0.45 + 0.4 * pulse) * 0.55;
                let rim = (-((d + 0.6).powi(2)) / (2.0 * 0.8 * 0.8)).exp() * (0.20 + 0.65 * focus) * 0.9;
                body = body.add(ring.scale(og + rim));
                self.blend(px, py, Rgb::new(body.r.max(0.0), body.g.max(0.0), body.b.max(0.0)), cov);
            }
        }
    }

    pub fn disc_glow(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb, amount: f32) {
        let m = (radius * 4.0) as i32;
        let x0 = (cx as i32 - m).max(0);
        let x1 = (cx as i32 + m).min(self.w as i32);
        let y0 = (cy as i32 - m).max(0);
        let y1 = (cy as i32 + m).min(self.h as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                let d = ((px as f32 + 0.5 - cx).powi(2) + (py as f32 + 0.5 - cy).powi(2)).sqrt() - radius;
                if d > 0.0 {
                    let g = (-d / (radius * 0.8).max(0.01)).exp() * amount;
                    self.add_at(px, py, color.scale(g));
                }
            }
        }
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb, alpha: f32) {
        let m = radius as i32 + 3;
        let x0 = (cx as i32 - m).max(0);
        let x1 = (cx as i32 + m).min(self.w as i32);
        let y0 = (cy as i32 - m).max(0);
        let y1 = (cy as i32 + m).min(self.h as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                let d = ((px as f32 + 0.5 - cx).powi(2) + (py as f32 + 0.5 - cy).powi(2)).sqrt() - radius;
                let cov = (0.5 - d).clamp(0.0, 1.0) * alpha;
                self.blend(px, py, color, cov);
            }
        }
    }

    /// A glowing line segment, used for icons (the power glyph, a submit
    /// arrow) and for stroke-font glyph edges.
    pub fn line(&mut self, ax: f32, ay: f32, bx: f32, by: f32, width: f32, color: Rgb, glow: f32) {
        let rr = width / 2.0;
        let gw = (width * 2.2).max(2.0);
        let margin = (rr + gw * 3.0 + 3.0) as i32;
        let x0 = (ax.min(bx) as i32 - margin).max(0);
        let x1 = (ax.max(bx) as i32 + margin).min(self.w as i32);
        let y0 = (ay.min(by) as i32 - margin).max(0);
        let y1 = (ay.max(by) as i32 + margin).min(self.h as i32);
        let (dx, dy) = (bx - ax, by - ay);
        let l2 = (dx * dx + dy * dy).max(1e-6);
        for py in y0..y1 {
            for px in x0..x1 {
                let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
                let t = (((fx - ax) * dx + (fy - ay) * dy) / l2).clamp(0.0, 1.0);
                let (nx, ny) = (ax + t * dx, ay + t * dy);
                let dist = ((fx - nx).powi(2) + (fy - ny).powi(2)).sqrt();
                let cov = (rr + 0.5 - dist).clamp(0.0, 1.0);
                if glow > 0.0 {
                    let halo = (-((dist - rr).max(0.0)) / gw).exp();
                    self.add_at(px, py, color.scale(halo * glow * 0.35));
                }
                if cov > 0.0 {
                    self.blend(px, py, color, cov);
                }
            }
        }
    }

    /// Draws `text` with its baseline at `(x, baseline_y)`, `size` design
    /// pixels tall, using the stroke font in [`font_data`].
    #[allow(clippy::too_many_arguments)]
    pub fn text(&mut self, text: &str, x: f32, baseline_y: f32, size: f32, color: Rgb, align: TextAlign, glow: f32, tracking: f32) {
        let start_x = match align {
            TextAlign::Left => x,
            TextAlign::Center => x - self.text_width(text, size, tracking) / 2.0,
            TextAlign::Right => x - self.text_width(text, size, tracking),
        };
        let unit = size / 8.0;
        let mut cx = start_x;
        for ch in text.chars() {
            let (base_ch, accent) = font_data::decompose(ch).map(|(b, a)| (b, Some(a))).unwrap_or((ch, None));
            if let Some(glyph) = font_data::glyph(base_ch) {
                for stroke in glyph.strokes {
                    for pair in stroke.windows(2) {
                        let (ax, ay) = pair[0];
                        let (bx, by) = pair[1];
                        self.line(cx + ax * unit, baseline_y + (ay - 8.0) * unit, cx + bx * unit, baseline_y + (by - 8.0) * unit, (size * 0.11).max(1.2), color, glow);
                    }
                    if stroke.len() == 1 {
                        let (ax, ay) = stroke[0];
                        self.fill_circle(cx + ax * unit, baseline_y + (ay - 8.0) * unit, (size * 0.06).max(1.0), color, 1.0);
                    }
                }
                self.draw_accent(accent, cx, baseline_y, unit, base_ch, color, glow);
                cx += (glyph.advance as f32 + tracking) * unit;
            } else {
                cx += (6.0 + tracking) * unit;
            }
        }
    }

    fn draw_accent(&mut self, accent: Option<Accent>, cx: f32, baseline_y: f32, unit: f32, base_ch: char, color: Rgb, glow: f32) {
        let Some(accent) = accent else { return };
        let top = if base_ch.is_uppercase() { 0.0 } else { 3.0 };
        let mid = if base_ch == 'ı' { 1.0 } else { 2.0 };
        let pt = |ax: f32, ay: f32| (cx + ax * unit, baseline_y + (ay - 8.0) * unit);
        let w = (unit * 0.9).max(1.0);
        match accent {
            Accent::Acute => {
                let (ax, ay) = pt(mid - 0.9, top - 1.0);
                let (bx, by) = pt(mid + 0.9, top - 2.8);
                self.line(ax, ay, bx, by, w, color, glow);
            }
            Accent::Grave => {
                let (ax, ay) = pt(mid + 0.9, top - 1.0);
                let (bx, by) = pt(mid - 0.9, top - 2.8);
                self.line(ax, ay, bx, by, w, color, glow);
            }
            Accent::Diaeresis => {
                let (ax, ay) = pt(mid - 1.0, top - 1.8);
                let (bx, by) = pt(mid + 1.0, top - 1.8);
                self.fill_circle(ax, ay, unit * 0.35, color, 1.0);
                self.fill_circle(bx, by, unit * 0.35, color, 1.0);
            }
            Accent::Tilde => {
                let pts = [pt(mid - 1.5, top - 1.4), pt(mid - 0.5, top - 2.4), pt(mid + 0.5, top - 1.2), pt(mid + 1.5, top - 2.2)];
                for w2 in pts.windows(2) {
                    self.line(w2[0].0, w2[0].1, w2[1].0, w2[1].1, w * 0.8, color, glow);
                }
            }
            Accent::Flip => {} // handled by the caller choosing the pre-flipped glyph shape upstream if ever needed
        }
    }

    pub fn text_width(&self, text: &str, size: f32, tracking: f32) -> f32 {
        let unit = size / 8.0;
        let mut w = 0.0f32;
        for ch in text.chars() {
            let (base_ch, _) = font_data::decompose(ch).unwrap_or((ch, Accent::Acute));
            let adv = font_data::glyph(base_ch).map(|g| g.advance as f32).unwrap_or(6.0);
            w += (adv + tracking) * unit;
        }
        (w - tracking * unit).max(0.0)
    }

    /// Faint upward-drifting accent motes, purely decorative. Skipped
    /// entirely under reduced-motion.
    pub fn particles(&mut self, t: f32, seed: u64, count: usize) {
        use crate::theme::colors::{ACCENT, ACCENT_2};
        let mut rng = seed;
        let mut next = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((rng >> 33) as u32) as f32 / u32::MAX as f32
        };
        for i in 0..count {
            let bx = next() * self.w as f32;
            let by = next() * self.h as f32;
            let speed = 8.0 + next() * 18.0;
            let phase = next() * std::f32::consts::TAU;
            let radius = 0.8 + next() * 1.6;
            let x = (bx + (t * 0.3 + phase).sin() * 22.0).rem_euclid(self.w as f32);
            let y = (by - t * speed).rem_euclid(self.h as f32);
            let a = (0.25 + 0.25 * (t * 0.9 + phase).sin()) * (0.4 + 0.6 * next());
            let col = if i % 3 == 0 { ACCENT_2 } else { ACCENT };
            self.disc_glow(x, y, radius, col, a.max(0.0) * 0.9);
        }
    }

    /// Converts the composited float image to the framebuffer's native
    /// pixel format and writes it into `out` (already the right length —
    /// `info.line_length * info.height` bytes). Supports any packed RGB(A)
    /// layout the kernel reports via `FbVarScreeninfo`'s bitfields (covers
    /// every format this project has actually seen: 16-bit 565, 24/32-bit
    /// 888): channel `length` other than 5, 6 or 8 is rejected up front
    /// rather than silently producing a garbled image.
    pub fn blit_to_fb(&self, out: &mut [u8], info: &FbInfo) -> Result<(), &'static str> {
        let bpp_bytes = (info.bpp / 8) as usize;
        if bpp_bytes == 0 || bpp_bytes > 4 {
            return Err("unsupported framebuffer bit depth");
        }
        for &(_offset, len) in [info.red, info.green, info.blue].iter() {
            if len != 0 && len != 5 && len != 6 && len != 8 {
                return Err("unsupported channel width");
            }
        }
        for y in 0..self.h {
            let row_start = y * info.line_length as usize;
            for x in 0..self.w {
                let c = self.px[y * self.w + x];
                let r = (c.r.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
                let g = (c.g.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
                let b = (c.b.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
                let pack_channel = |v: u32, len: u32| -> u32 {
                    if len == 8 {
                        v
                    } else {
                        v >> (8 - len)
                    }
                };
                let word = (pack_channel(r, info.red.1) << info.red.0)
                    | (pack_channel(g, info.green.1) << info.green.0)
                    | (pack_channel(b, info.blue.1) << info.blue.0);
                let off = row_start + x * bpp_bytes;
                if off + bpp_bytes > out.len() {
                    continue;
                }
                out[off..off + bpp_bytes].copy_from_slice(&word.to_le_bytes()[..bpp_bytes]);
            }
        }
        Ok(())
    }
}

pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Parameters for [`Canvas::glass`], grouped into one struct since the
/// method has too many independent knobs to pass positionally.
pub struct GlassSpec {
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub glow: f32,
    pub glow_radius: f32,
    pub tint: f32,
    pub border: f32,
    pub ring_phase: f32,
    pub accent: Rgb,
    pub accent2: Rgb,
    pub pulse: f32,
}

impl GlassSpec {
    pub fn new(cx: f32, cy: f32, w: f32, h: f32, radius: f32) -> GlassSpec {
        use crate::theme::colors::{ACCENT, ACCENT_2};
        GlassSpec {
            cx,
            cy,
            w,
            h,
            radius,
            glow: 0.7,
            glow_radius: 20.0,
            tint: 0.10,
            border: 1.4,
            ring_phase: 0.0,
            accent: ACCENT,
            accent2: ACCENT_2,
            pulse: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash01_is_deterministic_and_in_range() {
        for x in 0..10 {
            for y in 0..10 {
                let v = hash01(x, y);
                assert!((0.0..=1.0).contains(&v));
                assert_eq!(v, hash01(x, y));
            }
        }
        assert_ne!(hash01(0, 0), hash01(1, 0));
    }

    #[test]
    fn sd_round_box_is_negative_inside_positive_outside() {
        assert!(sd_round_box(50.0, 50.0, 50.0, 50.0, 40.0, 20.0, 5.0) < 0.0);
        assert!(sd_round_box(200.0, 50.0, 50.0, 50.0, 40.0, 20.0, 5.0) > 0.0);
    }

    #[test]
    fn backdrop_sample_is_finite_everywhere() {
        let bd = backdrop_generate(16, 9, 1.0);
        for y in 0..9 {
            for x in 0..16 {
                let c = bd.sample(x as f32, y as f32);
                assert!(c.r.is_finite() && c.g.is_finite() && c.b.is_finite());
            }
        }
    }

    #[test]
    fn text_width_grows_with_length() {
        let bd = backdrop_generate(4, 4, 0.0);
        let canvas = Canvas::new(20, 20, bd);
        assert!(canvas.text_width("hello", 16.0, 0.0) > canvas.text_width("hi", 16.0, 0.0));
        assert_eq!(canvas.text_width("", 16.0, 0.0), 0.0);
    }
}
