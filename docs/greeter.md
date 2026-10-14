# The greeter UI

## Rendering

There is no GPU here: [`Canvas`](../src/greeter/renderer.rs) is a small
software rasterizer operating on an RGB `f32` buffer, blitted to the
framebuffer's native pixel format once per frame
(`Canvas::blit_to_fb`, which packs into whatever bit layout
`FbVarScreeninfo` reports -- 16-bit 565, 24/32-bit 888, generically).

The animated backdrop (`backdrop_generate`) is computed at a small
resolution (160px wide) and bilinearly upsampled to the real output size --
cheap enough to redo every frame for a continuous, if subtle, drift. Glass
panels (`Canvas::glass`) refract this same backdrop texture through a
signed-distance rounded-box field, add an outer glow in the accent
gradient, a bright rim line, and a top-left highlight -- the same
"Liquid Glass" language `mitos-gui` uses, implemented independently since
this process shares no code with that project.

Text is drawn with a custom stroke font
([`src/greeter/font_data.rs`](../src/greeter/font_data.rs), generated from
a small Python glyph-design script, not hand-typed) rather than a font
file -- zero dependencies, and large/high-contrast accessibility modes are
just parameter changes (`theme::typography::LARGE_TEXT_MULTIPLIER`) rather
than a second font asset.

## Loop

~30 fps, single-threaded: `poll(2)` on the IPC link and every grabbed input
device (10ms timeout), drain whatever is ready, redraw if the frame
interval has elapsed, repeat. A `Heartbeat` request goes to the daemon
every 5 seconds so a hung greeter is detectable (the daemon does not yet
act on a missed heartbeat beyond what a crashed/exited process already
triggers -- see `docs/architecture.md`'s roadmap).

## Preview without hardware

```
mitos-login preview out.ppm 1280x720 login
mitos-login preview out.ppm 1280x720 locked-out
mitos-login preview out.ppm 1280x720 power-menu
mitos-login preview out.ppm 1280x720 error
```

Renders exactly one frame to a `.ppm` file (view with any image tool that
reads PPM, or convert with `pnmtopng`/ImageMagick) -- no daemon, no root,
no real framebuffer. This is also how the design was reviewed during
development.

## Accessibility

[`AccessibilitySettings`](../src/accessibility/mod.rs) (high contrast,
large text, reduced motion, visible focus) are plain data, consumed by
`Theme` (colors/type scale) and the render/input loop (motion, focus
rings) -- see `docs/architecture.md`'s roadmap for what is and is not
implemented yet (screen reader support is the one real gap).

Keyboard navigation (`src/accessibility/keyboard_navigation.rs`) is a
single flat focus ring: user tiles, then the password field, submit,
session menu, language menu, power button, and back around -- Tab/Shift+Tab
or the arrow keys move through it; there is no mouse-only interaction
anywhere in the greeter.
