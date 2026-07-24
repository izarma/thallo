use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, Color32},
};

use crate::{
    engine::{
        CoreSystems, UiPassSystems,
        design_scale::DesignScale,
        screens::{Screen, desktop::DesktopTextures},
        scripted_events::{NewFileReceiving, UnlockState},
        system_apps::{OpenAlertEvent, SystemAlerts},
    },
    ui::{
        apps::{WINDOW_DESIGN_H, WINDOW_DESIGN_W, WINDOW_PAD_BOT, WINDOW_PAD_X},
        theme::{
            palette::{CONTENT_FONT_SIZE, apply_button_theme},
            widgets::{
                primitives::progress_bar,
                title_bar::{TitleBarAction, title_bar},
            },
        },
    },
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OpenAlerts>();
    app.add_observer(on_open_alert);
    app.add_systems(
        Update, // Update progress state
        update_alert_progress
            .run_if(in_state(Screen::Desktop))
            .in_set(CoreSystems::Logic),
    );
    app.add_systems(
        EguiPrimaryContextPass,
        show_popups
            .run_if(in_state(Screen::Desktop))
            .in_set(UiPassSystems::Render),
    );
}

fn show_popups(
    mut contexts: EguiContexts,
    mut open_alerts: ResMut<OpenAlerts>,
    mut unlock_state: ResMut<UnlockState>,
    textures: Res<DesktopTextures>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let ft_tex = textures.file_transfer_sheet;
    let error_tex = textures.error_popup;
    let window_frame = egui::Frame::NONE;
    let window_size = scale.px(WINDOW_DESIGN_W / 2.0, WINDOW_DESIGN_H / 2.0);
    for entry in open_alerts.alerts.iter_mut() {
        if !entry.is_open {
            continue;
        }
        let mut close_requested = false;
        bevy_egui::egui::Window::new(&entry.name)
            .open(&mut entry.is_open)
            .resizable(false)
            .fade_in(true)
            .frame(window_frame)
            .fixed_size(window_size)
            .collapsible(false)
            .title_bar(false)
            .default_pos(ctx.viewport_rect().center() - window_size * 0.5)
            .show(ctx, |ui| {
                let bg_rect = ui.max_rect();
                let min = bg_rect.min;
                let max = bg_rect.max;
                let center = bg_rect.center();

                // 1. Define the 4 screen areas (where to draw on the window)
                let top_left_rect = egui::Rect::from_min_max(min, center);
                let top_right_rect = egui::Rect::from_min_max(
                    egui::pos2(center.x, min.y),
                    egui::pos2(max.x, center.y),
                );
                let bottom_left_rect = egui::Rect::from_min_max(
                    egui::pos2(min.x, center.y),
                    egui::pos2(center.x, max.y),
                );
                let bottom_right_rect = egui::Rect::from_min_max(center, max);
                // Grabbing the outer 25% corners of the source image
                let uv_tl = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(0.25, 0.25));
                let uv_tr = egui::Rect::from_min_max(egui::pos2(0.75, 0.0), egui::pos2(1.0, 0.25));
                let uv_bl = egui::Rect::from_min_max(egui::pos2(0.0, 0.75), egui::pos2(0.25, 1.0));
                let uv_br = egui::Rect::from_min_max(egui::pos2(0.75, 0.75), egui::pos2(1.0, 1.0));
                ui.painter()
                    .image(textures.window, top_left_rect, uv_tl, egui::Color32::WHITE);
                ui.painter()
                    .image(textures.window, top_right_rect, uv_tr, egui::Color32::WHITE);
                ui.painter().image(
                    textures.window,
                    bottom_left_rect,
                    uv_bl,
                    egui::Color32::WHITE,
                );
                ui.painter().image(
                    textures.window,
                    bottom_right_rect,
                    uv_br,
                    egui::Color32::WHITE,
                );
                // Title Bar
                ui.style_mut().interaction.selectable_labels = false;

                match &entry.alert {
                    SystemAlerts::FileTransfer(download) => {
                        title_bar(ui, "ALERT", &scale, false, false);
                        if render_file_transfer_ui(ui, entry.elapsed, download, ft_tex, &scale) {
                            close_requested = true;
                        }
                    }
                    SystemAlerts::EncryptedError => {
                        if title_bar(ui, "ERROR", &scale, true, false) == TitleBarAction::Close {
                            close_requested = true;
                        }
                        egui::Frame::default()
                            .inner_margin(egui::Margin {
                                left: (scale.x * WINDOW_PAD_X) as i8,
                                right: (scale.x * WINDOW_PAD_X) as i8,
                                top: 0,
                                bottom: scale.py(WINDOW_PAD_BOT) as i8,
                            })
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    // error.png source size is 541 x 233
                                    let available_width = ui.available_width();
                                    let img_size = egui::vec2(
                                        available_width,
                                        available_width * (233.0 / 541.0),
                                    );
                                    ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                        error_tex, img_size,
                                    )));
                                    ui.label(
                                        egui::RichText::new("File/Folder is Encrypted")
                                            .size(scale.py(CONTENT_FONT_SIZE)),
                                    );
                                });
                            });
                    }
                }
            });
        if close_requested {
            entry.is_open = false;
        }
    }
    for entry in open_alerts.alerts.iter() {
        if !entry.is_open
            && let SystemAlerts::FileTransfer(download) = &entry.alert
            && entry.elapsed >= LOAD_DURATION
        {
            match download {
                NewFileReceiving::BruteForce => unlock_state.bruteforce = true,
                NewFileReceiving::NetRipper => unlock_state.netripper = true,
            }
        }
    }

    open_alerts.alerts.retain(|a| a.is_open);
    Ok(())
}

#[derive(Debug)]
struct AlertEntry {
    name: String,
    alert: SystemAlerts,
    is_open: bool,
    elapsed: f32,
}

#[derive(Resource, Default)]
pub struct OpenAlerts {
    alerts: Vec<AlertEntry>,
}

fn on_open_alert(ev: On<OpenAlertEvent>, mut open_alerts: ResMut<OpenAlerts>) {
    open_alerts.alerts.push(AlertEntry {
        name: ev.name.clone(),
        alert: ev.alert.clone(),
        is_open: true,
        elapsed: 0.0,
    });
}

const LOAD_DURATION: f32 = 2.5;

fn update_alert_progress(time: Res<Time>, mut open_alerts: ResMut<OpenAlerts>) {
    for entry in open_alerts.alerts.iter_mut() {
        if let SystemAlerts::FileTransfer(_) = entry.alert
            && entry.elapsed < LOAD_DURATION
        {
            entry.elapsed += time.delta_secs();
        }
    }
}

fn render_file_transfer_ui(
    ui: &mut egui::Ui,
    elapsed: f32,
    download: &NewFileReceiving,
    texture: egui::TextureId,
    scale: &DesignScale,
) -> bool {
    // Apply common scaled text styles to this popup.
    let font_size = scale.py(CONTENT_FONT_SIZE);
    let font = egui::FontId::proportional(font_size);
    let style = ui.style_mut();
    style
        .text_styles
        .insert(egui::TextStyle::Body, font.clone());
    style.text_styles.insert(egui::TextStyle::Button, font);

    let progress = (elapsed / LOAD_DURATION).clamp(0.0, 1.0);
    let is_done = progress >= 1.0;
    // Calculate the current frame based on time
    let frame_index = ((elapsed * ANIM_FPS) as usize) % SHEET_FRAMES;
    // Calculate UV coordinates for the current frame (Horizontal Strip)
    let frame_step = 1.0 / SHEET_FRAMES as f32;
    let uv_min = egui::pos2(frame_index as f32 * frame_step, 0.0);
    let uv_max = egui::pos2((frame_index + 1) as f32 * frame_step, 1.0);
    let uv_rect = egui::Rect::from_min_max(uv_min, uv_max);
    let mut close = false;

    let space_sm = scale.py(10.0);

    ui.vertical_centered(|ui| {
        ui.add_space(space_sm);
        let sprite_size = scale.px(FRAME_DESIGN_W, FRAME_DESIGN_H);
        ui.add(egui::Image::new(egui::load::SizedTexture::new(texture, sprite_size)).uv(uv_rect));

        ui.add_space(space_sm);
        // Status Text
        let title = if is_done {
            format!("TRANSFER COMPLETE : {:?}", download)
        } else {
            format!("TRANSFERRING {:?}", download)
        };
        ui.label(egui::RichText::new(title).strong().color(Color32::WHITE));

        ui.add_space(space_sm);

        // Progress Bar
        let bar_width = scale.px(300.0, 0.0).x;
        let bar_height = scale.py(12.0);
        progress_bar(ui, progress, bar_width, bar_height);

        if is_done {
            apply_button_theme(ui);
            if ui
                .add_sized(scale.px(140.0, 40.0), egui::Button::new("Finish"))
                .clicked()
            {
                close = true;
            }
        } else {
            ui.ctx().request_repaint(); // Keep animating
        }
    });
    close
}

const SHEET_FRAMES: usize = 6;
const FRAME_DESIGN_W: f32 = 512.0;
const FRAME_DESIGN_H: f32 = 128.0;
const ANIM_FPS: f32 = 12.0;
