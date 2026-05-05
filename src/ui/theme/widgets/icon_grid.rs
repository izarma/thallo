use bevy_egui::egui::{self};

use crate::{
    engine::{file_system::FileType, screens::desktop::IconTextures},
    ui::theme::widgets::primitives::truncate_label,
};

/// Design-space icon display size (at 1920×1080).
pub const ICON_DESIGN_SIZE: f32 = 128.0;
/// Extra horizontal padding around each icon cell at design resolution.
const ICON_CELL_PAD: f32 = 16.0;
const ICON_LABEL_MAX_CHARS: usize = 12;

/// Padding (in design-space px) inserted around the entire grid as an inner margin.
const GRID_MARGIN_DESIGN: f32 = 12.0;

/// Design-space spacing between grid columns and rows.
const SPACING_X_DESIGN: f32 = 24.0;
const SPACING_Y_DESIGN: f32 = 32.0;

/// Extra vertical space below the icon image reserved for the label row.
const LABEL_ROW_DESIGN: f32 = 48.0;

/// Label font size at design resolution.
const LABEL_FONT_DESIGN: f32 = 18.0;
/// Floor so labels never become unreadable on very small windows.
const LABEL_FONT_MIN: f32 = 9.0;

/// A single item displayed in the grid.
pub struct IconGridItem {
    pub id: String,
    pub label: String,
    pub icon: IconRef,
    pub is_encrypted: bool,
}

/// What the caller should do after [`show_icon_grid`] returns.
pub enum IconGridAction {
    None,
    Selected(String),
    Opened(String),
}

#[derive(Clone, Copy)]
pub struct IconRef {
    pub texture_id: egui::TextureId,
    pub uv: egui::Rect,
}

impl IconRef {
    pub fn unlocked(texture_id: egui::TextureId) -> Self {
        Self {
            texture_id,
            uv: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(0.5, 1.0)),
        }
    }
    pub fn locked(texture_id: egui::TextureId) -> Self {
        Self {
            texture_id,
            uv: egui::Rect::from_min_max(egui::pos2(0.5, 0.0), egui::pos2(1.0, 1.0)),
        }
    }
}

/// Renders a scrollable icon grid and returns the interaction that occurred this frame.
///
/// `icon_size` is the **display** size (square side length) in current window pixels.
/// Compute it from `DesignScale`: `scale.uniform() * ICON_DESIGN_SIZE`.
pub fn show_icon_grid(
    ui: &mut egui::Ui,
    grid_id: &str,
    items: &[IconGridItem],
    selected: &Option<String>,
    cols: usize,
    icon_size: f32,
) -> IconGridAction {
    let mut action = IconGridAction::None;
    // Scale factor relative to design resolution — drives all derived measurements.
    let ratio = icon_size / ICON_DESIGN_SIZE;

    let icon_vec = egui::Vec2::splat(icon_size);
    let cell_width = icon_size + ICON_CELL_PAD * ratio;
    let row_height = icon_size + LABEL_ROW_DESIGN * ratio;
    let spacing = egui::Vec2::new(SPACING_X_DESIGN * ratio, SPACING_Y_DESIGN * ratio);
    let margin = GRID_MARGIN_DESIGN * ratio;
    let font_size = (LABEL_FONT_DESIGN * ratio).max(LABEL_FONT_MIN);
    egui::Frame::new()
        .inner_margin(egui::Margin::same(margin as i8))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                ui.add_space(margin);
                egui::Grid::new(grid_id)
                    .num_columns(cols)
                    .min_col_width(cell_width)
                    .max_col_width(cell_width)
                    .min_row_height(row_height)
                    .spacing(spacing)
                    .show(ui, |ui| {
                        for (col_idx, item) in items.iter().enumerate() {
                            let is_selected = selected.as_deref() == Some(item.id.as_str());

                            ui.vertical_centered(|ui| {
                                ui.set_min_width(cell_width);

                                let btn = egui::Button::image(
                                    egui::Image::new(egui::load::SizedTexture::new(
                                        item.icon.texture_id,
                                        icon_vec,
                                    ))
                                    .uv(item.icon.uv),
                                )
                                .corner_radius(10.0)
                                .selected(is_selected)
                                .frame(is_selected)
                                .fill(if is_selected {
                                    egui::Color32::from_gray(100)
                                } else {
                                    egui::Color32::TRANSPARENT
                                });

                                let resp = ui.add(btn);

                                if item.is_encrypted {
                                    let painter = ui.painter();
                                    let r = resp.rect;

                                    let time = ui.input(|i| i.time) as f32;

                                    let hash = |y: f32, t: f32| -> f32 {
                                        ((y * 12.9898 + t * 78.233).sin() * 43758.5453)
                                            .fract()
                                            .abs()
                                    };

                                    // 3. Port the horizontal block-based distortion logic
                                    let block_size = 15.0;
                                    let slice_height = r.height() / block_size;
                                    let uv_slice_height = item.icon.uv.height() / block_size;
                                    let glitch_strength = 12.0 * ratio; // Scale effect by resolution

                                    // Draw the icon in multiple horizontal slices
                                    for i in 0..(block_size as usize) {
                                        let i_f = i as f32;
                                        let block_uv = i_f / block_size;

                                        // Random trigger for glitching a specific block at a specific time
                                        let rand = hash(block_uv, time.floor());
                                        let trigger = if rand > 0.85 { 1.0 } else { 0.0 };

                                        let mut shift = 0.0;
                                        if trigger > 0.5 {
                                            shift = (hash(block_uv, time) * 2.0 - 1.0)
                                                * glitch_strength;
                                        }

                                        // Calculate geometry and UV rects for this specific slice
                                        let slice_rect = egui::Rect::from_min_max(
                                            egui::pos2(
                                                r.min.x + shift,
                                                r.min.y + i_f * slice_height,
                                            ),
                                            egui::pos2(
                                                r.max.x + shift,
                                                r.min.y + (i_f + 1.0) * slice_height,
                                            ),
                                        );

                                        let uv_rect = egui::Rect::from_min_max(
                                            egui::pos2(
                                                item.icon.uv.min.x,
                                                item.icon.uv.min.y + i_f * uv_slice_height,
                                            ),
                                            egui::pos2(
                                                item.icon.uv.max.x,
                                                item.icon.uv.min.y + (i_f + 1.0) * uv_slice_height,
                                            ),
                                        );

                                        // 4. Color channel drift with noise
                                        let drift =
                                            (hash(block_uv * 50.0, time) - 0.5) * glitch_strength;

                                        // Draw Red drift slice
                                        painter.image(
                                            item.icon.texture_id,
                                            slice_rect.translate(egui::vec2(drift, 0.0)),
                                            uv_rect,
                                            egui::Color32::from_rgba_unmultiplied(255, 60, 60, 150),
                                        );
                                        // Draw Blue drift slice
                                        painter.image(
                                            item.icon.texture_id,
                                            slice_rect.translate(egui::vec2(-drift, 0.0)),
                                            uv_rect,
                                            egui::Color32::from_rgba_unmultiplied(60, 60, 255, 150),
                                        );
                                        // Draw the main texture slice on top
                                        painter.image(
                                            item.icon.texture_id,
                                            slice_rect,
                                            uv_rect,
                                            egui::Color32::from_rgba_unmultiplied(
                                                255, 255, 255, 200,
                                            ),
                                        );
                                    }

                                    // Request a repaint to keep the glitch animation running smoothly
                                    ui.ctx().request_repaint();
                                }

                                if resp.clicked() {
                                    if is_selected {
                                        action = IconGridAction::Opened(item.id.clone());
                                    } else {
                                        action = IconGridAction::Selected(item.id.clone());
                                    }
                                }

                                if is_selected {
                                    // Show the full name, word-wrapped within the cell.

                                    ui.label(egui::RichText::new(&item.label).size(font_size));
                                } else {
                                    let label = truncate_label(&item.label, ICON_LABEL_MAX_CHARS);

                                    ui.label(egui::RichText::new(label).size(font_size));
                                }
                            });

                            if (col_idx + 1) % cols == 0 {
                                ui.end_row();
                            }
                        }

                        if !items.is_empty() && items.len() % cols != 0 {
                            ui.end_row();
                        }
                    });
                ui.add_space(margin);
            });
        });

    action
}

pub(crate) fn icon_for_filetype(ft: &FileType, icons: &IconTextures, is_locked: bool) -> IconRef {
    let sheet = match ft {
        FileType::Folder(_) => icons.folder,
        FileType::Image(..) => icons.png,
        FileType::TextFile(_) => icons.txt,
    };
    if is_locked {
        IconRef::locked(sheet)
    } else {
        IconRef::unlocked(sheet)
    }
}
