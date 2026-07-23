use bevy::{
    audio::Volume,
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use bevy_egui::egui;

use crate::engine::design_scale::DesignScale;

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 5.0;
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
    scale: &DesignScale,
) {
    let heading_size = scale.py(40.0);
    let space_sm = scale.py(40.0);
    ui.vertical(|ui| {
        // Scale widget chrome and text to match design resolution
        {
            let style = ui.style_mut();
            let font_size = scale.py(crate::ui::theme::palette::CONTENT_FONT_SIZE);
            let font = egui::FontId::proportional(font_size);
            style
                .text_styles
                .insert(egui::TextStyle::Body, font.clone());
            style.text_styles.insert(egui::TextStyle::Button, font);
            let u = scale.uniform();
            let s = &mut style.spacing;
            s.slider_width = u * 200.0;
            s.interact_size.y = scale.py(28.0);
            s.icon_width = u * 20.0;
            s.icon_spacing = u * 6.0;
            s.combo_height = scale.py(200.0);
        }
        {
            let s = ui.spacing_mut();
            let u = scale.uniform(); // uniform scale — no distortion
            s.slider_width = u * 200.0; // design-space slider track width
            s.interact_size.y = scale.py(28.0); // row height for combos, checkboxes, radios
            s.icon_width = u * 20.0; // radio/checkbox box size
            s.icon_spacing = u * 6.0; // gap between box and label
            s.combo_height = scale.py(200.0); // max dropdown height
        }
        ui.add_space(space_sm);
        ui.label(egui::RichText::new("Master Volume").size(heading_size));
        ui.add_space(space_sm);
        let mut linear = global_volume.volume.to_linear();
        let slider = egui::Slider::new(&mut linear, MIN_VOLUME..=MAX_VOLUME);
        if ui.add(slider).changed() {
            global_volume.volume = Volume::Linear(linear);
        }
        ui.separator();
        ui.add_space(space_sm);
        ui.label(egui::RichText::new("Display Mode").size(heading_size));
        ui.radio_value(window_mode, WindowMode::Windowed, "Windowed");
        ui.radio_value(
            window_mode,
            WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            "Fullscreen (Borderless)",
        );
        ui.separator();
        ui.add_space(space_sm);
        if *window_mode == WindowMode::Windowed {
            ui.add_space(space_sm);
            ui.label(egui::RichText::new("Resolution").size(heading_size));
            ui.add_space(scale.x * 12.0);
            let current_w = resolution.physical_width();
            let current_h = resolution.physical_height();
            let current_label = RESOLUTIONS
                .iter()
                .find(|(w, h, _)| *w == current_w && *h == current_h)
                .map(|(_, _, label)| *label)
                .unwrap_or("Custom");
            egui::ComboBox::from_id_salt("resolution")
                .width(ui.available_width())
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    for (w, h, label) in RESOLUTIONS {
                        let is_selected = current_w == *w && current_h == *h;
                        if ui.selectable_label(is_selected, *label).clicked() {
                            resolution.set_physical_resolution(*w, *h);
                        }
                    }
                });
            ui.add_space(space_sm);

            ui.checkbox(decorations, "OS Window Decorations");
            ui.separator();
        }
    });
}
