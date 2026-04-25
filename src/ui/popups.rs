use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, Color32},
};

use crate::engine::{
    UiPassSystems,
    screens::Screen,
    system_apps::{OpenAlertEvent, SystemAlerts},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OpenAlerts>();
    app.add_observer(on_open_alert);
    app.add_systems(
        Update, // Update progress state
        update_alert_progress.run_if(in_state(Screen::Desktop)),
    );
    app.add_systems(
        EguiPrimaryContextPass,
        show_popups
            .run_if(in_state(Screen::Desktop))
            .in_set(UiPassSystems::Render),
    );
}

fn show_popups(mut contexts: EguiContexts, mut open_alerts: ResMut<OpenAlerts>) -> Result {
    let ctx = contexts.ctx_mut()?;

    for entry in open_alerts.alerts.iter_mut() {
        if !entry.is_open {
            continue;
        }
        bevy_egui::egui::Window::new(&entry.name)
            .open(&mut entry.is_open)
            .resizable(false)
            .collapsible(false)
            .anchor(
                bevy_egui::egui::Align2::CENTER_CENTER,
                bevy_egui::egui::vec2(0.0, 0.0),
            )
            .show(ctx, |ui| match &entry.alert {
                SystemAlerts::FileTransfer(name) => {
                    render_file_transfer_ui(ui, entry.elapsed, name.clone());
                }
                _ => {
                    ui.label("System notification received.");
                }
            });
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

const LOAD_DURATION: f32 = 10.0;

fn update_alert_progress(time: Res<Time>, mut open_alerts: ResMut<OpenAlerts>) {
    for entry in open_alerts.alerts.iter_mut() {
        if let SystemAlerts::FileTransfer(_) = entry.alert {
            if entry.elapsed < LOAD_DURATION {
                entry.elapsed += time.delta_secs();
            }
        }
    }
}

fn render_file_transfer_ui(ui: &mut egui::Ui, elapsed: f32, name: String) {
    let progress = (elapsed / LOAD_DURATION).clamp(0.0, 1.0);
    let is_done = progress >= 1.0;

    ui.vertical_centered(|ui| {
        ui.add_space(10.0);

        // Status Text
        let title = if is_done {
            "TRANSFER COMPLETE".to_string()
        } else {
            format!("TRANSFERRING {}", name)
        };
        ui.label(egui::RichText::new(title).strong().color(Color32::WHITE));

        ui.add_space(8.0);

        // Progress Bar
        let bar_size = egui::vec2(300.0, 12.0);
        let (rect, _) = ui.allocate_exact_size(bar_size, egui::Sense::hover());

        // Draw Background
        ui.painter()
            .rect_filled(rect, 2.0, Color32::from_rgb(30, 30, 40));

        // Draw Fill
        if progress > 0.0 {
            let mut fill_rect = rect;
            fill_rect.set_width(rect.width() * progress);
            let fill_color = if is_done {
                Color32::from_rgb(80, 200, 120)
            } else {
                Color32::from_rgb(200, 70, 70)
            };
            ui.painter().rect_filled(fill_rect, 2.0, fill_color);
        }

        ui.add_space(8.0);

        // Percentage
        ui.label(format!("{:.0}%", progress * 100.0));

        if is_done {
            if ui.button("Close").clicked() {
                // In a real scenario, you might want to trigger a close event here
            }
        } else {
            ui.ctx().request_repaint(); // Keep animating
        }
    });
}
