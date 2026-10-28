//! The "Liquid Glass" palette: an animated dark gradient backdrop with
//! coloured light blooming through it, glass panels that refract and tint
//! it, and a cyan-to-violet accent gradient used for every glow/rim/focus
//! effect. Values are linear-ish `0.0..1.0` floats composited in
//! [`crate::greeter::renderer`]; only converted to the framebuffer's native
//! pixel format at the very last step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const fn new(r: f32, g: f32, b: f32) -> Rgb {
        Rgb { r, g, b }
    }
    pub fn lerp(self, other: Rgb, t: f32) -> Rgb {
        Rgb::new(self.r + (other.r - self.r) * t, self.g + (other.g - self.g) * t, self.b + (other.b - self.b) * t)
    }
    pub fn scale(self, k: f32) -> Rgb {
        Rgb::new(self.r * k, self.g * k, self.b * k)
    }
    pub fn add(self, other: Rgb) -> Rgb {
        Rgb::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }
    pub fn mul(self, other: Rgb) -> Rgb {
        Rgb::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }
}

pub const BG_TOP: Rgb = Rgb::new(0.020, 0.028, 0.075);
pub const BG_BOTTOM: Rgb = Rgb::new(0.030, 0.018, 0.060);

/// Colour, strength, and drift-speed/phase multiplier for each ambient light
/// bloom in the backdrop.
pub struct Bloom {
    pub color: Rgb,
    pub strength: f32,
    pub speed: f32,
    pub sigma: f32,
}

pub const BLOOMS: [Bloom; 4] = [
    Bloom { color: Rgb::new(0.10, 0.38, 1.00), strength: 0.55, speed: 1.0, sigma: 0.26 },
    Bloom { color: Rgb::new(0.55, 0.16, 0.95), strength: 0.50, speed: 2.0, sigma: 0.31 },
    Bloom { color: Rgb::new(0.02, 0.70, 0.78), strength: 0.42, speed: 3.0, sigma: 0.36 },
    Bloom { color: Rgb::new(0.95, 0.22, 0.55), strength: 0.30, speed: 4.0, sigma: 0.41 },
];

pub const ACCENT: Rgb = Rgb::new(0.36, 0.82, 1.00);
pub const ACCENT_2: Rgb = Rgb::new(0.72, 0.42, 1.00);
pub const TEXT: Rgb = Rgb::new(0.95, 0.97, 1.00);
pub const DIM: Rgb = Rgb::new(0.70, 0.76, 0.90);
pub const WARN: Rgb = Rgb::new(1.00, 0.72, 0.30);
pub const DANGER: Rgb = Rgb::new(1.00, 0.40, 0.40);
pub const GOOD: Rgb = Rgb::new(0.45, 0.95, 0.60);

/// High-contrast accessibility variants: pure white text and a brighter,
/// more saturated accent so glow/rim effects still read clearly.
pub const TEXT_HIGH_CONTRAST: Rgb = Rgb::new(1.0, 1.0, 1.0);
pub const ACCENT_HIGH_CONTRAST: Rgb = Rgb::new(0.40, 0.95, 1.00);
