use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};
use std::f32::consts::TAU;

use crate::engine::{
    UiPassSystems, asset_tracking::LoadResource, design_scale::DesignScale, screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<MinigameAssets>()
        .init_resource::<ActiveMinigame>()
        .add_observer(on_minigame_trigger)
        .add_systems(
            EguiPrimaryContextPass,
            (
                cache_minigame_textures.run_if(
                    resource_exists::<MinigameAssets>.and(not(resource_exists::<MinigameTextures>)),
                ),
                render_minigame_overlay
                    .run_if(resource_exists::<MinigameTextures>)
                    .run_if(minigame_active),
            )
                .in_set(UiPassSystems::Render)
                .run_if(in_state(Screen::Desktop)),
        )
        .add_systems(
            Update,
            // Outside CoreSystems intentionally — must run while Pause(true).
            update_bruteforce_logic
                .run_if(minigame_active)
                .run_if(in_state(Screen::Desktop)),
        );
}

#[derive(Resource, Default)]
pub struct ActiveMinigame(pub Option<Minigame>);

pub struct Minigame {
    game_type: MinigameType,
    checkpoint: usize,
    pub brute_force: Option<BruteForceState>,
}

pub struct BruteForceState {
    pub filled_slots: [bool; 8],
    pub current_angle: f32,
    pub speed: f32, // Radians per second
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum MinigameType {
    BruteForce,
    NetRipper,
}

#[derive(Event, Debug, Clone)]
pub struct MinigameTrigger {
    pub checkpoint: usize,
    pub game_type: MinigameType,
}

fn on_minigame_trigger(ev: On<MinigameTrigger>, mut state: ResMut<ActiveMinigame>) {
    if state.0.is_none() {
        state.0 = Some(Minigame {
            game_type: ev.game_type,
            checkpoint: ev.checkpoint,
            brute_force: if ev.game_type == MinigameType::BruteForce {
                Some(BruteForceState {
                    filled_slots: [false; 8],
                    current_angle: 0.0,
                    speed: 2.0, // Starting speed
                })
            } else {
                None
            },
        });
    }
}

fn minigame_active(mg: Res<ActiveMinigame>) -> bool {
    mg.0.is_some()
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MinigameAssets {
    #[dependency]
    pub hack1: Handle<Image>,
    #[dependency]
    pub hack1_disk: Handle<Image>,
    #[dependency]
    pub hack2: Handle<Image>,
    #[dependency]
    pub hack2_lock: Handle<Image>,
}

impl FromWorld for MinigameAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hack1: assets.load("minigames/hack_1.png"),
            hack1_disk: assets.load("minigames/hack_1_disks-sheet.png"),
            hack2: assets.load("minigames/hack_2.png"),
            hack2_lock: assets.load("minigames/hack_2_lock.png"),
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct MinigameTextures {
    hack2: egui::TextureId,
    hack2_lock: egui::TextureId,
}

fn cache_minigame_textures(
    mut contexts: EguiContexts,
    assets: Res<MinigameAssets>,
    mut cmd: Commands,
) {
    cmd.insert_resource(MinigameTextures {
        hack2: contexts.add_image(EguiTextureHandle::Weak(assets.hack2.id())),
        hack2_lock: contexts.add_image(EguiTextureHandle::Weak(assets.hack2_lock.id())),
    });
}

fn render_minigame_overlay(
    mut contexts: EguiContexts,
    textures: Res<MinigameTextures>,
    active: Res<ActiveMinigame>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let screen = ctx.viewport_rect();
    let center = screen.center();

    match active.0.as_ref().map(|mg| mg.game_type) {
        Some(MinigameType::BruteForce) => {
            if let Some(state) = &active.0.as_ref().unwrap().brute_force {
                render_bruteforce(ctx, screen, center, &textures, state, &scale);
            }
        }
        Some(MinigameType::NetRipper) => {}
        None => {}
    }

    ctx.request_repaint();
    Ok(())
}

fn render_bruteforce(
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

            let mut draw_lock_at_angle = |angle: f32, is_filled: bool| {
                // Calculate position on the circle
                let pos =
                    center + egui::vec2(angle.cos() * orbit_radius, angle.sin() * orbit_radius);
                let lock_rect = egui::Rect::from_center_size(pos, lock_size);

                // The image needs to point towards the center.
                // If the original image is vertical, adding PI/2 rotates it outward correctly.
                let image_rotation = angle + std::f32::consts::PI / 2.0;

                let mut lock_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(lock_rect)
                        .sense(egui::Sense::hover()),
                );

                // Dim the lock if it's already filled
                let tint = if is_filled {
                    egui::Color32::DARK_GRAY
                } else {
                    egui::Color32::WHITE
                };

                lock_ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(
                        textures.hack2_lock,
                        lock_size,
                    ))
                    .rotate(image_rotation, egui::Vec2::splat(0.5))
                    .tint(tint),
                );
            };

            // 1. Draw all previously filled notches
            for (i, &is_filled) in state.filled_slots.iter().enumerate() {
                if is_filled {
                    let angle = (i as f32) * (TAU / 8.0);
                    draw_lock_at_angle(angle, true);
                }
            }

            // 2. Draw the active moving lock
            draw_lock_at_angle(state.current_angle, false);
        });
}

fn update_bruteforce_logic(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut active: ResMut<ActiveMinigame>,
) {
    let Some(minigame) = &mut active.0 else {
        return;
    };
    let Some(state) = &mut minigame.brute_force else {
        return;
    };

    // 1. Move the lock
    state.current_angle += state.speed * time.delta_secs();
    state.current_angle %= TAU;

    // 2. Handle Spacebar click
    if keys.just_pressed(KeyCode::Space) {
        let total_slots = 8.0;
        let slot_angle = TAU / total_slots;
        let margin = 0.35; // Tolerance in radians (adjust for difficulty)

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

            // REVERSE & SPEED UP: Common in these games to flip direction on any "hit"
            // If it was a deselection (was_filled == true), maybe don't speed up as much.
            let speed_inc = if was_filled { 0.1 } else { 0.3 };
            state.speed = -state.speed.signum() * (state.speed.abs() + speed_inc);

            // 3. Check Win Condition
            // We only win if all slots are filled (meaning we didn't just deselect one)
            if state.filled_slots.iter().all(|&f| f) {
                info!("Hacked successfully!");
                active.0 = None;
            }
        } else {
            // Optional: Penalty for missing completely (click outside margin)
            // e.g., state.speed *= 0.9; // Slow down slightly as a "stumble"
        }
    }
}
