use bevy::{
    audio::Volume,
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use bevy_egui::egui;

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 10.0;
const RESOLUTIONS: &[(u32, u32, &str)] = &[
    (2560, 1440, "2560×1440"),
    (1920, 1080, "1920×1080"),
    (1600, 900, "1600×900"),
    (1280, 720, "1280×720"),
    (1024, 768, "1024×768"),
    (1024, 576, "1024 × 576"),
];

pub fn show_settings_window(
    ui: &mut egui::Ui,
    global_volume: &mut GlobalVolume,
    window_mode: &mut WindowMode,
    resolution: &mut WindowResolution,
    decorations: &mut bool,
) {
    ui.add_space(12.0);
    ui.label("Settings");
    ui.separator();
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Master Volume");
        ui.add_space(12.0);

        let mut linear = global_volume.volume.to_linear();
        if ui
            .add(egui::Slider::new(&mut linear, MIN_VOLUME..=MAX_VOLUME).text("Volume"))
            .changed()
        {
            global_volume.volume = Volume::Linear(linear);
        }
    });

    ui.add_space(8.0);

    ui.label("Display Mode");

    ui.horizontal(|ui| {
        ui.radio_value(window_mode, WindowMode::Windowed, "Windowed");
        ui.radio_value(
            window_mode,
            WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            "Fullscreen (Borderless)",
        );
    });
    ui.add_space(8.0);

    if *window_mode == WindowMode::Windowed {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Resolution");
            ui.add_space(12.0);

            let current_w = resolution.physical_width();
            let current_h = resolution.physical_height();
            let current_label = RESOLUTIONS
                .iter()
                .find(|(w, h, _)| *w == current_w && *h == current_h)
                .map(|(_, _, label)| *label)
                .unwrap_or("Custom");

            egui::ComboBox::from_id_salt("resolution")
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    for (w, h, label) in RESOLUTIONS {
                        let is_selected = current_w == *w && current_h == *h;
                        if ui.selectable_label(is_selected, *label).clicked() {
                            resolution.set_physical_resolution(*w, *h);
                        }
                    }
                });
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.checkbox(decorations, "OS Window Decorations");
        });
    }

    ui.add_space(8.0);
}
