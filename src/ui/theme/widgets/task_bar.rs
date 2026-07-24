use bevy_egui::egui;

use crate::ui::theme::{
    palette::{BUTTON_ACTIVE_BG, BUTTON_BG, BUTTON_HOVERED_BG, HEADER_COLOR, apply_button_theme},
    widgets::primitives::truncate_label,
};

/// Design-space sizes at 1920×1080.  Callers scale these via `DesignScale`
/// before passing them in — the widget itself is resolution-agnostic.
pub const TAB_DESIGN_W: f32 = 226.0; // half of the 452-wide spritesheet
pub const START_DESIGN_W: f32 = 68.0;
pub const TASKBAR_DESIGN_H: f32 = 28.0;

const TASKBAR_LABEL_MAX_CHARS: usize = 16;
const TAB_UV_INACTIVE: egui::Rect =
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(0.5, 1.0));
const TAB_UV_ACTIVE: egui::Rect =
    egui::Rect::from_min_max(egui::pos2(0.5, 0.0), egui::pos2(1.0, 1.0));

/// Action returned by [`taskbar_app_button`].
pub enum TaskbarAppAction {
    None,
    /// The button was left-clicked (toggle minimize).
    Clicked,
    /// "Close" was selected from the context menu.
    Close,
}

/// A task-bar button representing an open application window.
///
/// `is_active` true when the window is visible (not minimised).
/// `tab_size`pass `scale.px(TAB_DESIGN_W, TAB_DESIGN_H)` from the calling system.
/// `font_size` already scaled with DesignScale
/// Returns a [`TaskbarAppAction`] indicating what the user did.
pub fn taskbar_app_button(
    ui: &mut egui::Ui,
    label: impl Into<String>,
    is_active: bool,
    icon: Option<egui::TextureId>,
    tab_size: egui::Vec2,
    font_size: f32,
) -> TaskbarAppAction {
    let mut action = TaskbarAppAction::None;
    let label: String = label.into();
    let display = truncate_label(&label, TASKBAR_LABEL_MAX_CHARS);
    let (rect, response) = ui.allocate_exact_size(tab_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let uv = if is_active {
            TAB_UV_ACTIVE
        } else {
            TAB_UV_INACTIVE
        };

        if let Some(tex_id) = icon {
            ui.painter().image(tex_id, rect, uv, egui::Color32::WHITE);
        } else {
            // Solid-colour fallback while the spritesheet hasn't loaded yet.
            let fallback_color = if is_active {
                BUTTON_ACTIVE_BG
            } else if response.hovered() {
                BUTTON_HOVERED_BG
            } else {
                BUTTON_BG
            };
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(4), fallback_color);
        }
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            display,
            egui::FontId::proportional(font_size),
            BUTTON_BG,
        );
    }

    if response.clicked() {
        action = TaskbarAppAction::Clicked;
    }
    if label != "Chat" {
        response.context_menu(|ui| {
            if ui.button("Close").clicked() {
                action = TaskbarAppAction::Close;
                ui.close();
            }
        });
    }

    action
}

/// Action returned by [`taskbar_group_button`].
pub enum GroupTabAction {
    None,
    /// A specific window in the group was left-clicked (toggle minimize).
    Selected(egui::Id),
    /// The × button on a specific window row was clicked.
    Close(egui::Id),
    /// "Close All" was chosen from the right-click context menu.
    CloseAll,
}

/// A single window entry passed to [`taskbar_group_button`].
pub struct GroupedWindow {
    pub id: egui::Id,
    pub name: String,
    pub is_minimized: bool,
}

pub fn taskbar_group_button(
    ui: &mut egui::Ui,
    label: impl Into<String>,
    is_active: bool,
    icon: Option<egui::TextureId>,
    tab_size: egui::Vec2,
    windows: &[GroupedWindow],
    font_size: f32,
) -> GroupTabAction {
    let label: String = label.into();
    let count = windows.len();
    let display = truncate_label(&format!("{} ({})", label, count), TASKBAR_LABEL_MAX_CHARS);

    let (rect, response) = ui.allocate_exact_size(tab_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        // Highlight the button while its popup is open.
        let group_popup_id = response.id.with("group_popup");
        let popup_open =
            egui::Popup::is_id_open(ui.ctx(), group_popup_id) || response.context_menu_opened();
        let draw_active = is_active || popup_open;
        paint_tab_bg(ui, rect, &response, draw_active, icon);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            display,
            egui::FontId::proportional(font_size),
            BUTTON_BG,
        );
    }

    let mut action = GroupTabAction::None;

    // Popup::menu handles click-to-toggle and CloseOnClick automatically.
    // The taskbar sits at the bottom of the screen so egui's auto-align
    // will open the menu above it.
    // Use a dedicated ID for the left-click popup so it doesn't collide
    // with the right-click context menu (which uses the response's default ID).
    egui::Popup::menu(&response)
        .id(response.id.with("group_popup"))
        .width(tab_size.x)
        .show(|ui| {
            ui.set_min_width(tab_size.x);
            for win in windows {
                let row_label = truncate_label(&win.name, TASKBAR_LABEL_MAX_CHARS);
                let text_color = if win.is_minimized {
                    egui::Color32::from_gray(160)
                } else {
                    HEADER_COLOR
                };

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let close_width = tab_size.y;

                    // Main row: click to toggle minimize.
                    apply_button_theme(ui);
                    let btn = ui.add_sized(
                        [tab_size.x - close_width, tab_size.y],
                        egui::Button::new(
                            egui::RichText::new(row_label)
                                .size(font_size)
                                .color(text_color),
                        ),
                    );
                    if btn.clicked() {
                        // Menu closes itself on click (CloseOnClick is Popup::menu default).
                        action = GroupTabAction::Selected(win.id);
                    }

                    // Small × button to close just this window.
                    apply_button_theme(ui);
                    let close_btn = ui.add_sized(
                        [close_width, tab_size.y],
                        egui::Button::new(egui::RichText::new("×").size(font_size)),
                    );
                    if close_btn.clicked() {
                        action = GroupTabAction::Close(win.id);
                        ui.close();
                    }
                });
            }
        });

    // Right-click: context menu with bulk actions.
    response.context_menu(|ui| {
        if ui.button("Close All").clicked() {
            action = GroupTabAction::CloseAll;
            ui.close();
        }
    });

    action
}

/// A slim full-width menu row button, suitable for start-menu style lists.
/// Returns the [`egui::Response`] so the caller can check `.clicked()`.
pub fn menu_item(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    tab_size: egui::Vec2,
    font_size: f32,
) -> egui::Response {
    apply_button_theme(ui);
    ui.add_sized(
        tab_size,
        egui::Button::new(
            egui::RichText::new(text)
                .size(font_size)
                .color(HEADER_COLOR),
        )
        .right_text("")
        .corner_radius(egui::CornerRadius::same(4)),
    )
}

fn paint_tab_bg(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    is_active: bool,
    icon: Option<egui::TextureId>,
) {
    let uv = if is_active {
        TAB_UV_ACTIVE
    } else {
        TAB_UV_INACTIVE
    };
    if let Some(tex_id) = icon {
        ui.painter().image(tex_id, rect, uv, egui::Color32::WHITE);
    } else {
        let fallback_color = if is_active {
            BUTTON_ACTIVE_BG
        } else if response.hovered() {
            BUTTON_HOVERED_BG
        } else {
            BUTTON_BG
        };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(4), fallback_color);
    }
}
