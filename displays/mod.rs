//! The framebuffer output. mitos-login draws through the kernel's legacy
//! `/dev/fb0` interface rather than DRM/KMS directly: on virtually every
//! real system this device already exists (`CONFIG_DRM_FBDEV_EMULATION`,
//! on by default, backs it even on a pure KMS driver, already mode-set to
//! the boot resolution) and needs no mode enumeration or CRTC setup at all.
//! Multi-output layouts and direct KMS are explicitly **not** implemented —
//! see docs/architecture.md's roadmap section — this targets the common
//! single-output case correctly rather than half-covering the general one.
pub mod display;
pub mod layout;
pub mod manager;
pub mod resolution;

pub use display::Display;
pub use manager::DisplayManager;
