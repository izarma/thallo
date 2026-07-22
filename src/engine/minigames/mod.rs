use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::engine::{
    UiPassSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    design_scale::DesignScale,
    minigames::{
        bruteforce::{BruteForceState, render_bruteforce, update_bruteforce_logic},
        netripper::{NetRipperState, render_netripper, update_netripper_logic},
    },
    screens::Screen,
    scripted_events::ScriptedEventTrigger,
    system_apps::Applications,
    window_manager::OpenWindows,
};

mod bruteforce;
mod netripper;

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
            (update_bruteforce_logic, update_netripper_logic)
                .run_if(minigame_active)
                .run_if(in_state(Screen::Desktop)),
        );
    app.add_observer(handle_minigame_outcome);
}

#[derive(Resource, Default)]
pub struct ActiveMinigame(pub Option<Minigame>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum MinigameType {
    BruteForce,
    NetRipper,
}

#[derive(Event, Debug, Clone)]
pub struct MinigameTrigger {
    pub game_type: MinigameType,
}

pub struct Minigame {
    game_type: MinigameType,
    pub brute_force: Option<BruteForceState>,
    pub net_ripper: Option<NetRipperState>,
}

#[derive(Event, Debug, Clone)]
pub struct MinigameOutcome {
    pub game_type: MinigameType,
    pub success: bool,
}

fn on_minigame_trigger(ev: On<MinigameTrigger>, mut state: ResMut<ActiveMinigame>) {
    if state.0.is_none() {
        let (brute_force, net_ripper) = match ev.game_type {
            MinigameType::BruteForce => (
                Some(BruteForceState {
                    filled_slots: [false; 8],
                    current_angle: 0.0,
                    speed: 2.0,
                    missed_timer: 0.0,
                    miss_count: 0,
                    failed: false,
                }),
                None,
            ),
            MinigameType::NetRipper => (None, Some(NetRipperState::new_random())),
        };

        state.0 = Some(Minigame {
            game_type: ev.game_type,
            brute_force,
            net_ripper,
        });
    }
}

fn minigame_active(mg: Res<ActiveMinigame>) -> bool {
    mg.0.is_some()
}

#[derive(Resource, Clone, Copy)]
struct MinigameTextures {
    hack1: egui::TextureId,
    hack2: egui::TextureId,
    hack2_lock: egui::TextureId,
}

fn cache_minigame_textures(
    mut contexts: EguiContexts,
    assets: Res<MinigameAssets>,
    mut cmd: Commands,
) {
    cmd.insert_resource(MinigameTextures {
        hack1: contexts.add_image(EguiTextureHandle::Weak(assets.hack1.id())),
        hack2: contexts.add_image(EguiTextureHandle::Weak(assets.hack2.id())),
        hack2_lock: contexts.add_image(EguiTextureHandle::Weak(assets.hack2_lock.id())),
    });
}

fn render_minigame_overlay(
    mut contexts: EguiContexts,
    textures: Res<MinigameTextures>,
    mut active: ResMut<ActiveMinigame>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let screen = ctx.viewport_rect();
    let center = screen.center();

    if let Some(mg) = &mut active.0 {
        match mg.game_type {
            MinigameType::BruteForce => {
                if let Some(state) = &active.0.as_ref().unwrap().brute_force {
                    render_bruteforce(ctx, screen, center, &textures, state, &scale);
                }
            }
            MinigameType::NetRipper => {
                if let Some(state) = &mut mg.net_ripper {
                    render_netripper(ctx, screen, center, state, &scale, &textures);
                }
            }
        }
    }
    ctx.request_repaint();
    Ok(())
}

fn handle_minigame_outcome(
    ev: On<MinigameOutcome>,
    mut open_windows: ResMut<OpenWindows>,
    mut cmd: Commands,
    assets: Res<MinigameAssets>,
) {
    let outcome = ev.event();
    if outcome.success {
        // Successful minigames clear their own overlay, and the app timer resumes automatically.
        cmd.spawn(sound_effect(assets.hack_win.clone()));
    } else {
        cmd.spawn(sound_effect(assets.hack_lose.clone()));
        // Apply fail states to the relevant open window
        for window in open_windows.windows.iter_mut() {
            match &mut window.event.app_type {
                // BRUTEFORCE FAIL: Close the decrypter entirely
                Applications::Decrypter { .. } if ev.game_type == MinigameType::BruteForce => {
                    info!("Decryption failed! Closing window.");
                    window.is_open = false;
                }

                // NETRIPPER FAIL: Check max tries, either reset progress or trigger Game Over
                Applications::Ripper {
                    max_tries,
                    elapsed,
                    minigames_triggered,
                    ..
                } if ev.game_type == MinigameType::NetRipper => {
                    if let Some(tries) = max_tries {
                        *tries = tries.saturating_sub(1);

                        if *tries == 0 {
                            info!("NetRipper max tries reached. Triggering Game Over!");
                            cmd.trigger(ScriptedEventTrigger::RipperFailed);
                            window.is_open = false;
                            continue;
                        }
                    }

                    info!("NetRipper failed! Resetting progress.");
                    // Resetting to 0.0 restarts the transmission attempt in the app.
                    *elapsed = 0.0;
                    *minigames_triggered = 0;
                }
                _ => {}
            }
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MinigameAssets {
    #[dependency]
    hack1: Handle<Image>,
    #[dependency]
    hack2: Handle<Image>,
    #[dependency]
    hack2_lock: Handle<Image>,
    #[dependency]
    pub hack_win: Handle<AudioSource>,
    #[dependency]
    pub hack_lose: Handle<AudioSource>,
}

impl FromWorld for MinigameAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hack1: assets.load("minigames/hack_1.png"),
            hack2: assets.load("minigames/hack_2.png"),
            hack2_lock: assets.load("minigames/hack_2_lock.png"),
            hack_win: assets.load("audio/sfx/hack_success.ogg"),
            hack_lose: assets.load("audio/sfx/hack_error.ogg"),
        }
    }
}
