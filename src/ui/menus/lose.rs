use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::{
        design_scale::DesignScale,
        screens::Screen,
        video::{VideoPlayer, VideoPlayers, cutscene_finished, spawn_fullscreen_video},
    },
    ui::{menus::Menu, theme::widgets::primitives},
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
    mut players: NonSendMut<VideoPlayers>,
) {
    spawn_fullscreen_video(
        &mut commands,
        &mut images,
        &mut players,
        DEATH_VIDEO_PATH,
        false,
        "LoseVideo",
        Menu::Lose,
    );
}

fn lose_menu(
    mut contexts: EguiContexts,
    scale: Res<DesignScale>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut app_exit: MessageWriter<AppExit>,
    video: Query<&VideoPlayer>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Hide the menu until the death cutscene has finished playing.
    if !cutscene_finished(&video) {
        return Ok(());
    }

    primitives::centered_panel(ctx, "lose_menu", |ui| {
        primitives::header(ui, "CONNECTION LOST", &scale);
        primitives::label(
            ui,
            "The catastrophe arrived before you could send the SOS.",
            &scale,
        );
        if primitives::button(ui, "Retry", &scale).clicked() {
            next_screen.set(Screen::ActBreak);
        }
        if primitives::button(ui, "Quit", &scale).clicked() {
            app_exit.write(AppExit::Success);
        }
    });
    Ok(())
}
