use bevy::prelude::*;

use crate::engine::{
    asset_tracking::ResourceHandles,
    cursor::CurrentCursor,
    screens::{
        Screen,
        desktop::{DesktopAssets, DesktopTextures},
    },
    video::{
        VideoAudioSource, VideoPlayer, VideoPlayers, cutscene_finished, spawn_fullscreen_video,
    },
};

const BOOTUP_VIDEO_PATH: &str = "assets/cutscenes/bootup.mp4";

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_startup);
    app.add_systems(
        Update,
        enter_desktop_screen.run_if(
            in_state(Screen::Loading)
                .and(is_startup_done)
                .and(resource_exists::<DesktopTextures>),
        ),
    );
}

fn spawn_startup(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    mut audio_sources: ResMut<Assets<VideoAudioSource>>,
    mut players: NonSendMut<VideoPlayers>,
    assets: Res<AssetServer>,
    mut current: ResMut<CurrentCursor>,
) {
    current.handle = assets.load("ui/cursors/cursor_loading.png");
    spawn_fullscreen_video(
        &mut cmd,
        &mut images,
        &mut audio_sources,
        &mut players,
        BOOTUP_VIDEO_PATH,
        false,
        true,
        "BootupVideo",
        Screen::Loading,
    );
}

// checks if all assets are loaded and if the bootup cutscene has finished playing
fn is_startup_done(resource_handles: Res<ResourceHandles>, video: Query<&VideoPlayer>) -> bool {
    resource_handles.is_all_done() && cutscene_finished(&video)
}

fn enter_desktop_screen(
    mut next_screen: ResMut<NextState<Screen>>,
    mut current: ResMut<CurrentCursor>,
    d_ass: Res<DesktopAssets>,
) {
    info!("Entering desktop screen");
    next_screen.set(Screen::Desktop);
    current.handle = d_ass.main_cursor.clone();
}
