use bevy_egui::egui;

use crate::engine::design_scale::DesignScale;
use crate::ui::theme::palette::{
    CONTENT_FONT_SIZE, HEADER_COLOR, HEADING_FONT_SIZE, LABEL_COLOR, RED_CONTRAST_THEME,
    RED_CONTRAST_THEME2, apply_button_theme,
};

/// Wraps `body` in a vertically-and-horizontally centred [`egui::CentralPanel`]
pub fn centered_panel(ctx: &egui::Context, id: &str, body: impl FnOnce(&mut egui::Ui)) {
    // consume the CentralPanel so egui doesn't complain about unused space
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |_ui| {});

    egui::Area::new(egui::Id::new(id))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 20.0;
            ui.vertical_centered(|ui| {
                body(ui);
            });
        });
}

/// Horizontally centred panel positioned 25% below the screen centre.
pub fn low_centered_panel(ctx: &egui::Context, id: &str, body: impl FnOnce(&mut egui::Ui)) {
    // consume the CentralPanel so egui doesn't complain about unused space
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |_ui| {});

    // egui screen co-ordinates have +y pointing down, so a positive y offset
    // moves the panel lower on screen (25% of the screen height below centre).
    let offset = egui::vec2(0.0, ctx.content_rect().height() * 0.25);

    egui::Area::new(egui::Id::new(id))
        .anchor(egui::Align2::CENTER_CENTER, offset)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 20.0;
            ui.vertical_centered(|ui| {
                body(ui);
            });
        });
}

/// for startup
pub fn startup_panel(ctx: &egui::Context, id: &str, body: impl FnOnce(&mut egui::Ui)) {
    // consume the CentralPanel so egui doesn't complain about unused space
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |_ui| {});

    let rect = ctx.content_rect();
    // +y moves down, so a positive y offset pushes the panel lower.
    // Negative x moves it left of center.
    let offset = egui::vec2(-rect.width() * 0.17, rect.height() * 0.20);

    egui::Area::new(egui::Id::new(id))
        .anchor(egui::Align2::CENTER_CENTER, offset)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 20.0;
            ui.vertical_centered(|ui| {
                body(ui);
            });
        });
}

/// A large header label (≈ 40 px).
pub fn header(ui: &mut egui::Ui, text: impl Into<String>, scale: &DesignScale) {
    ui.label(
        egui::RichText::new(text)
            .size(HEADING_FONT_SIZE * scale.x)
            .color(HEADER_COLOR)
            .strong(),
    );
}

/// A regular text label (≈ 24 px).
pub fn label(ui: &mut egui::Ui, text: impl Into<String>, scale: &DesignScale) -> egui::Response {
    ui.label(
        egui::RichText::new(text)
            .size(CONTENT_FONT_SIZE * scale.x)
            .color(LABEL_COLOR),
    )
}

const BUTTON_FRAME_W: f32 = 1.0 / 3.0;
const BUTTON_IDLE_UV: egui::Rect =
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(BUTTON_FRAME_W, 1.0));
const BUTTON_HOVER_UV: egui::Rect = egui::Rect::from_min_max(
    egui::pos2(BUTTON_FRAME_W, 0.0),
    egui::pos2(BUTTON_FRAME_W * 2.0, 1.0),
);
const BUTTON_PRESSED_UV: egui::Rect =
    egui::Rect::from_min_max(egui::pos2(BUTTON_FRAME_W * 2.0, 0.0), egui::pos2(1.0, 1.0));

/// Native design size of one frame of `ui/button.png`.
const BUTTON_NATIVE_SIZE: egui::Vec2 = egui::vec2(175.0, 40.0);
/// Width of the rounded cap at each end of the button, in asset pixels.
const BUTTON_CAP_W: f32 = 20.0;

/// Picks the sprite-sheet frame for the given interaction state.  A `selected`
/// toggle keeps the Pressed frame until the pointer interacts with it again.
fn button_uv(response: &egui::Response, selected: bool) -> egui::Rect {
    if response.is_pointer_button_down_on() {
        BUTTON_PRESSED_UV
    } else if response.hovered() {
        BUTTON_HOVER_UV
    } else if selected {
        BUTTON_PRESSED_UV
    } else {
        BUTTON_IDLE_UV
    }
}

/// Draws one sprite frame into `rect`, falling back to a themed rounded rect
/// while the asset is still loading.
fn paint_button_frame(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    texture: Option<egui::TextureId>,
    uv: egui::Rect,
) {
    if let Some(texture) = texture {
        ui.painter().image(texture, rect, uv, egui::Color32::WHITE);
    } else {
        // Fallback while the asset is still loading.
        apply_button_theme(ui);
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(40),
            ui.visuals().widgets.inactive.weak_bg_fill,
        );
    }
}

/// Draws a horizontally-stretched button frame into `rect`, keeping the rounded
/// caps at the native aspect and stretching the centre.  Falls back to a
/// themed rounded rect while the asset is still loading.
fn paint_sliced_button_frame(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    texture: Option<egui::TextureId>,
    uv: egui::Rect,
) {
    if let Some(texture) = texture {
        // Scale the cap width with the target height so the ends keep their
        // original 20:40 proportions.
        let cap_w = (rect.height() * (BUTTON_CAP_W / BUTTON_NATIVE_SIZE.y))
            .min(rect.width() * 0.5)
            .max(0.0);
        let uv_cap = (BUTTON_CAP_W / BUTTON_NATIVE_SIZE.x) * uv.width();

        let left_uv = egui::Rect::from_min_max(uv.min, egui::pos2(uv.min.x + uv_cap, uv.max.y));
        let right_uv = egui::Rect::from_min_max(egui::pos2(uv.max.x - uv_cap, uv.min.y), uv.max);
        let mid_uv = egui::Rect::from_min_max(
            egui::pos2(uv.min.x + uv_cap, uv.min.y),
            egui::pos2(uv.max.x - uv_cap, uv.max.y),
        );

        let left_rect = egui::Rect::from_min_size(rect.min, egui::vec2(cap_w, rect.height()));
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(rect.max.x - cap_w, rect.min.y),
            egui::vec2(cap_w, rect.height()),
        );
        let mid_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + cap_w, rect.min.y),
            egui::pos2(rect.max.x - cap_w, rect.max.y),
        );

        ui.painter()
            .image(texture, left_rect, left_uv, egui::Color32::WHITE);
        if mid_rect.width() > 0.0 {
            ui.painter()
                .image(texture, mid_rect, mid_uv, egui::Color32::WHITE);
        }
        ui.painter()
            .image(texture, right_rect, right_uv, egui::Color32::WHITE);
    } else {
        // Fallback while the asset is still loading.
        apply_button_theme(ui);
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(40),
            ui.visuals().widgets.inactive.weak_bg_fill,
        );
    }
}

/// A button rendered with the `ui/button.png` three-frame sprite (Idle, Hovered,
/// Pressed).  Each frame is 175 × 40; the image is drawn at that native design
/// size and scaled by [`DesignScale`].  Returns the [`egui::Response`] so the
/// caller can check `.clicked()`, `.hovered()`, etc.
pub fn button(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    scale: &DesignScale,
    texture: Option<egui::TextureId>,
) -> egui::Response {
    let text = text.into();
    let size = scale.px(175.0, 40.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        paint_button_frame(ui, rect, texture, button_uv(&response, false));

        let text_color = if response.hovered() {
            Color32::BLACK
        } else {
            HEADER_COLOR
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(scale.py(16.0)),
            text_color,
        );
    }

    response
}

/// A variable-width button rendered with the `ui/button.png` three-frame sprite.
/// The rounded caps keep their native 20:40 proportions and the centre is
/// stretched to fill `size`.  `text_color` is used for the idle state; the text
/// turns black while hovered.
pub fn sliced_button(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    size: egui::Vec2,
    font_size: f32,
    text_color: egui::Color32,
    texture: Option<egui::TextureId>,
) -> egui::Response {
    let text = text.into();
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        paint_sliced_button_frame(ui, rect, texture, button_uv(&response, false));

        let color = if response.hovered() {
            egui::Color32::BLACK
        } else {
            text_color
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(font_size),
            color,
        );
    }

    response
}

/// An icon-only button rendered with a three-frame sprite sheet (Idle, Hovered,
/// Pressed), e.g. `ui/reveal_button.png` (40 × 40 frames in a 120 × 40 sheet).
/// The image is drawn at `frame_size` and scaled by [`DesignScale`]; `selected`
/// pins it to the Pressed frame for a toggle look.
pub fn icon_button(
    ui: &mut egui::Ui,
    scale: &DesignScale,
    frame_size: egui::Vec2,
    texture: Option<egui::TextureId>,
    selected: bool,
) -> egui::Response {
    let size = scale.px(frame_size.x, frame_size.y);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        paint_button_frame(ui, rect, texture, button_uv(&response, selected));
    }

    response
}

/// A placeholder label for empty or error states — italicised and dimmed.
pub fn empty_state(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(text).italics().weak());
    });
}

/// Truncates a label to `max_chars` codepoints, appending `…` if cut.
/// Uses `chars().count()` so multi-byte characters (emoji, CJK) are counted correctly.
pub fn truncate_label(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        format!("{}…", s.chars().take(max_chars - 1).collect::<String>())
    } else {
        s.to_string()
    }
}

use bevy_egui::egui::Color32;

pub fn progress_bar(ui: &mut egui::Ui, progress: f32, width: f32, height: f32) {
    let progress = progress.clamp(0.0, 1.0);
    let done = progress >= 1.0;

    let (outer, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let p = ui.painter();

    // Border — deep red when done, contrast red while in progress
    let border = if done {
        RED_CONTRAST_THEME
    } else {
        RED_CONTRAST_THEME2
    };
    p.rect_filled(outer, egui::CornerRadius::same(3), border);

    // Dark track
    let inner = outer.shrink(1.0);
    p.rect_filled(
        inner,
        egui::CornerRadius::same(2),
        Color32::from_rgb(14, 14, 22),
    );

    // Fill — deep red when done, contrast red while in progress
    if progress > 0.0 {
        let fill = if done {
            RED_CONTRAST_THEME2
        } else {
            RED_CONTRAST_THEME
        };
        let fill_rect = egui::Rect::from_min_size(
            inner.min,
            egui::vec2(inner.width() * progress, inner.height()),
        );
        p.rect_filled(fill_rect, egui::CornerRadius::same(2), fill);
    }
    ui.label(format!("{:.0}%", progress * 100.0));
}
