use crate::{
    engine::{
        design_scale::DesignScale,
        minigames::{ActiveMinigame, Minigame, MinigameOutcome, MinigameTextures, MinigameType},
    },
    ui::theme::palette::RED_CONTRAST_THEME,
};
use bevy::prelude::*;
use bevy_egui::egui;
use rand::seq::SliceRandom;
use std::f32::consts::{FRAC_PI_2, TAU};

const SLOT_COUNT: usize = 8;
const MAX_MISSES: u8 = 10;
const NORTH_ANGLE: f32 = -FRAC_PI_2;

// Notches are numbered clockwise from north. Every configuration must contain each notch once.
const BRUTEFORCE_CONFIGS: &[&[&[usize]]] = &[
    &[&[0, 1, 2], &[4, 5, 6], &[3, 7]],
    &[&[0, 3, 4], &[1, 5, 6], &[2, 7]],
    &[&[0, 2, 4, 6], &[1, 3, 5, 7]],
    &[&[0, 4], &[1, 5], &[2, 6], &[3, 7]],
    &[&[0, 1, 7], &[2, 3], &[4, 5, 6]],
];

#[derive(Resource, Default)]
pub(super) struct BruteForceConfigBag {
    remaining: Vec<usize>,
    last_drawn: Option<usize>,
}

impl BruteForceConfigBag {
    fn draw(&mut self, rng: &mut impl rand::Rng) -> usize {
        if self.remaining.is_empty() {
            self.remaining = (0..BRUTEFORCE_CONFIGS.len()).collect();
            self.remaining.shuffle(rng);

            // Avoid repeating the previous bag's final config immediately after a refill.
            if self.remaining.len() > 1 && self.remaining.last() == self.last_drawn.as_ref() {
                self.remaining.swap(0, BRUTEFORCE_CONFIGS.len() - 1);
            }
        }

        let drawn = self
            .remaining
            .pop()
            .expect("bruteforce configs are not empty");
        self.last_drawn = Some(drawn);
        drawn
    }
}

pub struct BruteForceState {
    filled_slots: [bool; SLOT_COUNT],
    active_sets: Vec<Vec<usize>>,
    current_set: usize,
    current_angle: f32,
    speed: f32,        // Radians per second
    missed_timer: f32, // Time remaining for the "red" flash
    miss_count: u8,
    failed: bool,
}

impl BruteForceState {
    pub(super) fn new_random(config_bag: &mut BruteForceConfigBag) -> Self {
        let mut rng = rand::rng();
        let config_index = config_bag.draw(&mut rng);
        let mut active_sets: Vec<Vec<usize>> = BRUTEFORCE_CONFIGS[config_index]
            .iter()
            .map(|set| set.to_vec())
            .collect();
        active_sets.shuffle(&mut rng);

        Self {
            filled_slots: [false; SLOT_COUNT],
            active_sets,
            current_set: 0,
            current_angle: NORTH_ANGLE,
            speed: 2.0,
            missed_timer: 0.0,
            miss_count: 0,
            failed: false,
        }
    }

    fn is_open(&self, slot: usize) -> bool {
        self.active_sets[self.current_set].contains(&slot)
    }

    fn current_set_complete(&self) -> bool {
        self.active_sets[self.current_set]
            .iter()
            .all(|&slot| self.filled_slots[slot])
    }
}

fn notch_angle(index: usize) -> f32 {
    NORTH_ANGLE + (index as f32) * (TAU / SLOT_COUNT as f32)
}

pub(super) fn render_bruteforce(
    ctx: &egui::Context,
    screen: egui::Rect,
    center: egui::Pos2,
    textures: &MinigameTextures,
    state: &BruteForceState,
    scale: &DesignScale,
) {
    let display_scale = 1.5;
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

            // Highlight only the notches that are open in the current set.
            for &slot in &state.active_sets[state.current_set] {
                if !state.filled_slots[slot] {
                    let angle = notch_angle(slot);
                    let pos =
                        center + egui::vec2(angle.cos() * orbit_radius, angle.sin() * orbit_radius);
                    painter.circle_filled(
                        pos,
                        8.0 * scale.uniform() * display_scale,
                        RED_CONTRAST_THEME,
                    );
                    painter.circle_stroke(
                        pos,
                        12.0 * scale.uniform() * display_scale,
                        egui::Stroke::new(2.0 * scale.uniform(), RED_CONTRAST_THEME),
                    );
                }
            }

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
                    draw_lock_at_angle(notch_angle(i), true, false);
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
    mut cmd: Commands,
) {
    let mut clear_active = false;
    let Some(Minigame::BruteForce(state)) = &mut active.0 else {
        return;
    };
    if state.missed_timer > 0.0 {
        state.missed_timer -= time.delta_secs();
    }

    // Move the lock
    state.current_angle = (state.current_angle + state.speed * time.delta_secs()).rem_euclid(TAU);

    // Handle input
    if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
        let slot_angle = TAU / SLOT_COUNT as f32;
        let margin = 0.125; // Tolerance in radians (adjust for difficulty)

        let normalized = state.current_angle.rem_euclid(TAU);

        // Notches are indexed clockwise from north.
        let closest_idx =
            ((normalized - NORTH_ANGLE).rem_euclid(TAU) / slot_angle).round() as usize % SLOT_COUNT;
        let target_angle = notch_angle(closest_idx).rem_euclid(TAU);

        // Calculate shortest angular distance
        let mut diff = (normalized - target_angle).abs();
        if diff > std::f32::consts::PI {
            diff = TAU - diff;
        }

        if diff <= margin && state.is_open(closest_idx) && !state.filled_slots[closest_idx] {
            state.filled_slots[closest_idx] = true;
            state.speed = -state.speed.signum() * (state.speed.abs() + 0.3);

            if state.current_set_complete() {
                if state.current_set + 1 == state.active_sets.len() {
                    info!("Hacked successfully!");
                    cmd.trigger(MinigameOutcome {
                        game_type: MinigameType::BruteForce,
                        success: true,
                    });
                    clear_active = true;
                } else {
                    // Draw the next set from this run's already-shuffled set bag.
                    state.current_set += 1;
                }
            }
        } else {
            // Missing a notch or choosing one that is not currently open is a mistake.
            state.missed_timer = 0.2;
            state.miss_count += 1;
            state.speed = -state.speed;
        }
    } // Check fail condition
    if !clear_active && state.miss_count > MAX_MISSES {
        state.failed = true;
        cmd.trigger(MinigameOutcome {
            game_type: MinigameType::BruteForce,
            success: false,
        });
        // Flag the minigame to be cleared
        clear_active = true;
    }
    if clear_active {
        active.0 = None;
    }
}
