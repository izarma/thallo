use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::{
    engine::{
        audio::sound_effect,
        design_scale::DesignScale,
        screens::Screen,
        scripted_events::{StoryBeat, UnlockState},
        video::{VideoAudioSource, VideoPlayers, spawn_fullscreen_video},
    },
    game::FileAssets,
    ui::{menus::Menu, theme::widgets::primitives},
};

const DISCLAIMER_VIDEO_PATH: &str = "assets/cutscenes/disclaimer.mp4";

/// Marks the Act 2 break sound effect so the title card can wait for it to finish.
#[derive(Component)]
struct ActBreakSfx;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Startup), spawn_startup_video);
    app.add_systems(OnEnter(Screen::ActBreak), spawn_actbreak_sfx);
    app.add_systems(
        EguiPrimaryContextPass,
        startup_ui.run_if(in_state(Menu::Startup)),
    );
    app.add_systems(
        EguiPrimaryContextPass,
        act_break_title.run_if(in_state(Screen::ActBreak)),
    );
}

fn spawn_startup_video(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut audio_sources: ResMut<Assets<VideoAudioSource>>,
    mut players: NonSendMut<VideoPlayers>,
) {
    spawn_fullscreen_video(
        &mut commands,
        &mut images,
        &mut audio_sources,
        &mut players,
        DISCLAIMER_VIDEO_PATH,
        true,
        false,
        "StartupVideo",
        Menu::Startup,
    );
}

fn startup_ui(
    mut contexts: EguiContexts,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    scale: Res<DesignScale>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    primitives::low_centered_panel(ctx, "startup_menu", |ui| {
        ui.style_mut().interaction.selectable_labels = false;

        if primitives::button(ui, "Yes", &scale).clicked() {
            info!("enter load - yes click");
            next_screen.set(Screen::Loading);
        }

        if primitives::button(ui, "No", &scale).clicked() {
            next_menu.set(Menu::ShutDown);
        }
    });

    Ok(())
}

/// The single act-break title card. Both narrative transitions (`Act1Connecting`
/// and `Act2Sos`) now land here directly, instead of first showing a header-only
/// interstitial that auto-advances to `Screen::Title`.
fn act_break_title(
    mut contexts: EguiContexts,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    scale: Res<DesignScale>,
    state: Res<UnlockState>,
    assets: Res<FileAssets>,
    act_break_sfx: Query<(), With<ActBreakSfx>>,
) -> Result {
    let act_transition_texture =
        contexts.add_image(EguiTextureHandle::Weak(assets.act_transition.id()));
    let ctx = contexts.ctx_mut()?;
    match state.story.beat {
        StoryBeat::Act1Connecting => {
            primitives::centered_panel(ctx, "reconnect_menu", |ui| {
                primitives::header(ui, "SUNDAY NETWORK", &scale);
                primitives::label(ui, "Terminal reconnected. Incoming sync detected.", &scale);
                if primitives::button(ui, "Connect", &scale).clicked() {
                    next_screen.set(Screen::Loading);
                    next_menu.set(Menu::None);
                }
            });
        }
        StoryBeat::Act2Sos => {
            ctx.layer_painter(egui::LayerId::background()).image(
                act_transition_texture,
                ctx.content_rect(),
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            primitives::low_centered_panel(ctx, "act2_title", |ui| {
                if act_break_sfx.is_empty() && primitives::button(ui, "Power On", &scale).clicked()
                {
                    next_screen.set(Screen::Loading);
                    next_menu.set(Menu::None);
                }
            });
        }
        StoryBeat::Act1Intro | StoryBeat::Win | StoryBeat::Lose => {}
    }
    Ok(())
}

fn spawn_actbreak_sfx(mut cmd: Commands, assets: Res<FileAssets>, state: Res<UnlockState>) {
    if state.story.should_play_break_sfx() {
        info!("break sfx playing");
        cmd.spawn((sound_effect(assets.tans_act.clone()), ActBreakSfx));
    }
}
