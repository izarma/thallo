use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::egui;

/// Native resolution every asset was authored at.
pub const DESIGN_WIDTH: f32 = 1920.0;
pub const DESIGN_HEIGHT: f32 = 1080.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DesignScale>();
    app.add_systems(PreUpdate, update_design_scale);
}

/// Ratio between the current window size and the 1920×1080 design resolution.
///
/// Updated once per frame by [`update_design_scale`].  Read by any UI system
/// that needs to convert a design-space measurement into window-space pixels:
///
/// ```rust
/// // In a Bevy system:
/// let tab_size = scale.px(226.0, 28.0);  // non-uniform (stretches with window)
/// let icon_size = scale.uniform() * 64.0; // uniform  (no distortion)
/// ```
#[derive(Resource, Clone, Copy, Debug)]
pub struct DesignScale {
    /// `window_width  / DESIGN_WIDTH`
    pub x: f32,
    /// `window_height / DESIGN_HEIGHT`
    pub y: f32,
}

impl Default for DesignScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

impl DesignScale {
    pub fn from_window(logical_w: f32, logical_h: f32) -> Self {
        Self {
            x: logical_w / DESIGN_WIDTH,
            y: logical_h / DESIGN_HEIGHT,
        }
    }

    /// Convert a design-space (width, height) pair to current window pixels.
    /// Use this for assets whose X and Y should track the window independently
    /// (e.g. the taskbar, which must always fill the full width and exact height).
    #[inline]
    pub fn px(&self, design_w: f32, design_h: f32) -> egui::Vec2 {
        egui::vec2(design_w * self.x, design_h * self.y)
    }

    /// Uniform (minimum) scale.  Use for square/icon assets so they are never
    /// stretched — they will be as large as possible without exceeding either axis.
    #[inline]
    pub fn uniform(&self) -> f32 {
        self.x.min(self.y)
    }

    /// Scale a single value along the Y axis (heights, font sizes tied to vertical space).
    #[inline]
    pub fn py(&self, design_h: f32) -> f32 {
        design_h * self.y
    }

    // /// Scale a single value along the X axis.
    // #[inline]
    // pub fn pxa(&self, design_w: f32) -> f32 {
    //     design_w * self.x
    // }
}

fn update_design_scale(
    mut scale: ResMut<DesignScale>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(win) = window.single() else { return };
    let new = DesignScale::from_window(win.width(), win.height());
    // Only write when something actually changed to avoid spurious change-detection.
    if (new.x - scale.x).abs() > f32::EPSILON || (new.y - scale.y).abs() > f32::EPSILON {
        *scale = new;
    }
}
