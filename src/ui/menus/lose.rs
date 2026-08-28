use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

use crate::{
    engine::{
        design_scale::DesignScale,
        screens::Screen,
        scripted_events::StoryBeat,
        video::{
            VideoAudioSource, VideoPlayer, VideoPlayers, cutscene_finished, spawn_fullscreen_video,
        },
    },
    game::{FileAssets, beats::ApplyBeatCommand},
    ui::{
        menus::Menu,
        theme::{button_textures::ButtonTextures, widgets::primitives},
    },
};

const DEATH_VIDEO_PATH: &str = "assets/cutscenes/death_screen.mp4";

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Lose), spawn_lose_video);
    app.add_systems(
        EguiPrimaryContextPass,
        lose_menu.run_if(in_state(Menu::Lose)),
    );
}

fn spawn_lose_video(
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
        DEATH_VIDEO_PATH,
        false,
        true,
        "LoseVideo",
        Menu::Lose,
    );
}

fn lose_menu(
    mut contexts: EguiContexts,
    scale: Res<DesignScale>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut cmd: Commands,
    mut app_exit: MessageWriter<AppExit>,
    video: Query<&VideoPlayer>,
    button_textures: Option<Res<ButtonTextures>>,
    assets: Res<FileAssets>,
) -> Result {
    let button_texture = button_textures.as_deref().map(|t| t.button);
    let background = contexts.add_image(EguiTextureHandle::Weak(assets.lose.id()));
    let ctx = contexts.ctx_mut()?;

    // Hide the menu until the death cutscene has finished playing.
    if !cutscene_finished(&video) {
        return Ok(());
    }

    ctx.layer_painter(egui::LayerId::background()).image(
        background,
        ctx.content_rect(),
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    primitives::low_centered_panel(ctx, "lose_menu", |ui| {
        if primitives::button(ui, "Retry", &scale, button_texture).clicked() {
            // Rebuild a clean Act 2 and boot straight back to the desktop,
            // skipping the ActBreak -> Title -> Act2Startup title card.
            cmd.queue(ApplyBeatCommand(StoryBeat::Act2Sos));
            next_menu.set(Menu::None);
            next_screen.set(Screen::Loading);
        }
        if primitives::button(ui, "Quit", &scale, button_texture).clicked() {
            app_exit.write(AppExit::Success);
        }
    });
    Ok(())
}
