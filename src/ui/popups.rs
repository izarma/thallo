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
    ui::theme::widgets::primitives::progress_bar,
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
    let texture_id = textures.file_transfer_sheet;
    for entry in open_alerts.alerts.iter_mut() {
        if !entry.is_open {
            continue;
        }
        let mut close_requested = false;
        bevy_egui::egui::Window::new(&entry.name)
            .open(&mut entry.is_open)
            .resizable(false)
            .collapsible(false)
            .anchor(
                bevy_egui::egui::Align2::CENTER_CENTER,
                bevy_egui::egui::vec2(0.0, 0.0),
            )
            .show(ctx, |ui| match &entry.alert {
                SystemAlerts::FileTransfer(download) => {
                    if render_file_transfer_ui(ui, entry.elapsed, download, texture_id, &scale) {
                        close_requested = true;
                    }
                }
                SystemAlerts::EncryptedError => {
                    ui.label("File/Folder is Encrypted");
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

    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        let sprite_size = scale.px(FRAME_DESIGN_W, FRAME_DESIGN_H);
        ui.add(egui::Image::new(egui::load::SizedTexture::new(texture, sprite_size)).uv(uv_rect));

        ui.add_space(scale.py(10.0));
        // Status Text
        let title = if is_done {
            "TRANSFER COMPLETE".to_string()
        } else {
            format!("TRANSFERRING {:?}", download)
        };
        ui.label(egui::RichText::new(title).strong().color(Color32::WHITE));

        ui.add_space(8.0);

        // Progress Bar
        progress_bar(ui, progress, 300.0, 12.0);

        if is_done {
            if ui.button("Finish").clicked() {
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
