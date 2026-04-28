//! Animation utilities — easing functions and interpolation.
//!
//! All easing functions take `t` in [0.0, 1.0] and return a curved value
//! in [0.0, 1.0]. Combine with `lerp` for smooth transitions.
#![allow(dead_code)]

/// Linear interpolation between two values.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Starts fast, decelerates to rest. Good for things arriving.
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Smooth acceleration then deceleration. Good for position changes.
pub fn ease_in_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Starts slow, accelerates. Good for things departing.
pub fn ease_in_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t
}

/// Animation duration constants (in seconds).
pub const GAUGE_SWEEP_DURATION: f32 = 0.6;
pub const TOAST_FADE_DURATION: f32 = 0.5;
pub const TAB_SLIDE_DURATION: f32 = 0.2;
pub const COUNT_UP_DURATION: f32 = 0.4;

/// Tick delta for 60fps animation frames.
pub const TICK_DELTA: f32 = 1.0 / 60.0;
