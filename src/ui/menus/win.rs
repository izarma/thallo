use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::{
        design_scale::DesignScale,
        video::{
            VideoAudioSource, VideoPlayer, VideoPlayers, cutscene_finished, spawn_fullscreen_video,
        },
    },
    ui::{
        menus::Menu,
        theme::{button_textures::ButtonTextures, widgets::primitives},
    },
};

const ENDING_VIDEO_PATH: &str = "assets/cutscenes/thallo_ending.mp4";

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Win), spawn_win_video);
    app.add_systems(EguiPrimaryContextPass, win_menu.run_if(in_state(Menu::Win)));
}

fn spawn_win_video(
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
        ENDING_VIDEO_PATH,
        false,
        true,
        "WinVideo",
        Menu::Win,
    );
}

fn win_menu(
    mut contexts: EguiContexts,
    scale: Res<DesignScale>,
    mut app_exit: MessageWriter<AppExit>,
    video: Query<&VideoPlayer>,
    button_textures: Option<Res<ButtonTextures>>,
) -> Result {
    let button_texture = button_textures.as_deref().map(|t| t.button);
    let ctx = contexts.ctx_mut()?;

    // Hide the menu until the ending cutscene has finished playing.
    if !cutscene_finished(&video) {
        return Ok(());
    }

    primitives::centered_panel(ctx, "win_menu", |ui| {
        primitives::header(ui, "TRANSMISSION RECEIVED", &scale);
        primitives::label(ui, "something something", &scale);
        if primitives::button(ui, "Quit", &scale, button_texture).clicked() {
            app_exit.write(AppExit::Success);
        }
    });
    Ok(())
}
