use crate::engine::{
    design_scale::DesignScale,
    minigames::{ActiveMinigame, MinigameTextures},
};
use bevy::prelude::*;
use bevy_egui::egui;
use std::f32::consts::TAU;

pub struct BruteForceState {
    pub filled_slots: [bool; 8],
    pub current_angle: f32,
    pub speed: f32,        // Radians per second
    pub missed_timer: f32, // Time remaining for the "red" flash
}

pub(super) fn render_bruteforce(
    ctx: &egui::Context,
    screen: egui::Rect,
    center: egui::Pos2,
    textures: &MinigameTextures,
    state: &BruteForceState,
    scale: &DesignScale,
) {
    let display_scale = 2.0;
    let bg_size = scale.px(512.0, 512.0) * display_scale;
    let lock_size = egui::vec2(
        22.0 * scale.uniform() * display_scale,
        44.0 * scale.uniform() * display_scale,
    );

    // The radius from the center where the locks should orbit
    // Adjust the 0.3 multiplier to align exactly with your background sprite's notches
    let orbit_radius = bg_size.x * 0.30;

    egui::Area::new(egui::Id::new("mg_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::Pos2::ZERO)
        .interactable(true)
        .show(ctx, |ui| {
            let (_, painter) = ui.allocate_painter(screen.size(), egui::Sense::click_and_drag());
            // 1. Scrim
            painter.rect_filled(
                screen,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(200),
            );

            // 2. hack2 background — centred, scales with both axes independently.
            let bg_rect = egui::Rect::from_center_size(center, bg_size);
            painter.image(
                textures.hack2,
                bg_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            let mut draw_lock_at_angle = |angle: f32, is_filled: bool, is_active: bool| {
                // Calculate position on the circle
                let pos =
                    center + egui::vec2(angle.cos() * orbit_radius, angle.sin() * orbit_radius);
                let lock_rect = egui::Rect::from_center_size(pos, lock_size);
                let image_rotation = angle + std::f32::consts::PI / 2.0;

                // Dim the lock if it's already filled
                let mut tint = if is_filled {
                    egui::Color32::DARK_GRAY
                } else {
                    egui::Color32::WHITE
                };
                if is_active && state.missed_timer > 0.0 {
                    tint = egui::Color32::DARK_RED;
                }

                let mut lock_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(lock_rect)
                        .sense(egui::Sense::hover()),
                );

                lock_ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(
                        textures.hack2_lock,
                        lock_size,
                    ))
                    .rotate(image_rotation, egui::Vec2::splat(0.5))
                    .tint(tint),
                );
            };

            // Draw all previously filled notches
            for (i, &is_filled) in state.filled_slots.iter().enumerate() {
                if is_filled {
                    let angle = (i as f32) * (TAU / 8.0);
                    draw_lock_at_angle(angle, true, false);
                }
            }

            // 2. Draw the active moving lock
            draw_lock_at_angle(state.current_angle, false, true);
        });
}

pub(super) fn update_bruteforce_logic(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut active: ResMut<ActiveMinigame>,
) {
    let Some(minigame) = &mut active.0 else {
        return;
    };
    let Some(state) = &mut minigame.brute_force else {
        return;
    };
    if state.missed_timer > 0.0 {
        state.missed_timer -= time.delta_secs();
    }

    // Move the lock
    state.current_angle += state.speed * time.delta_secs();
    state.current_angle %= TAU;

    // Handle input
    if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
        let total_slots = 8.0;
        let slot_angle = TAU / total_slots;
        let margin = 0.25; // Tolerance in radians (adjust for difficulty)

        let normalized = state.current_angle.rem_euclid(TAU);

        // Find the index of the closest notch
        let closest_idx = (normalized / slot_angle).round() as usize % 8;
        let target_angle = (closest_idx as f32) * slot_angle;

        // Calculate shortest angular distance
        let mut diff = (normalized - target_angle).abs();
        if diff > std::f32::consts::PI {
            diff = TAU - diff;
        }

        if diff <= margin {
            // Capture the state before toggling for logic checks
            let was_filled = state.filled_slots[closest_idx];

            // TOGGLE: This fills it if empty, and deselects it if already filled.
            state.filled_slots[closest_idx] = !was_filled;
            if was_filled {
                state.missed_timer = 0.5;
            }

            // REVERSE & SPEED UP
            // If it was a deselection (was_filled == true), maybe don't speed up as much.
            let speed_inc = if was_filled { 0.1 } else { 0.3 };
            state.speed = -state.speed.signum() * (state.speed.abs() + speed_inc);

            // Check Win Condition
            // We only win if all slots are filled (meaning we didn't just deselect one)
            if state.filled_slots.iter().all(|&f| f) {
                info!("Hacked successfully!");
                active.0 = None;
            }
        } else {
            // Optional: Penalty for missing completely (click outside margin)
            state.missed_timer = 0.2;
            state.speed = -state.speed;
        }
    }
}
