use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::{
        audio::sound_effect,
        design_scale::DesignScale,
        screens::Screen,
        scripted_events::{StoryBeat, UnlockState},
        video::{VideoPlayers, spawn_fullscreen_video},
    },
    game::FileAssets,
    ui::{menus::Menu, theme::widgets::primitives},
};

const DISCLAIMER_VIDEO_PATH: &str = "assets/cutscenes/disclaimer.mp4";

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
    mut players: NonSendMut<VideoPlayers>,
) {
    spawn_fullscreen_video(
        &mut commands,
        &mut images,
        &mut players,
        DISCLAIMER_VIDEO_PATH,
        true,
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
) -> Result {
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
            primitives::centered_panel(ctx, "act2_title", |ui| {
                primitives::header(ui, "ACT II", &scale);
                primitives::label(ui, "5 minutes before the catastrophe on Sunday", &scale);
                if primitives::button(ui, "Power On", &scale).clicked() {
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
        cmd.spawn(sound_effect(assets.tans_act.clone()));
    }
}
