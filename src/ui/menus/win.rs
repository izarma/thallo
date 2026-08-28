use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::{
    engine::{
        design_scale::DesignScale,
        video::{
            VideoAudioSource, VideoPlayer, VideoPlayers, cutscene_finished, spawn_fullscreen_video,
        },
    },
    game::FileAssets,
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
    assets: Res<FileAssets>,
) -> Result {
    let button_texture = button_textures.as_deref().map(|t| t.button);
    let background = contexts.add_image(EguiTextureHandle::Weak(assets.win.id()));
    let ctx = contexts.ctx_mut()?;

    // Hide the menu until the ending cutscene has finished playing.
    if !cutscene_finished(&video) {
        return Ok(());
    }

    ctx.layer_painter(egui::LayerId::background()).image(
        background,
        ctx.content_rect(),
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    primitives::low_centered_panel(ctx, "win_menu", |ui| {
        if primitives::button(ui, "Quit", &scale, button_texture).clicked() {
            app_exit.write(AppExit::Success);
        }
    });
    Ok(())
}
