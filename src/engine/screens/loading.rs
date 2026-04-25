use bevy::{
    prelude::*,
    window::{CursorIcon, CustomCursor, CustomCursorImage},
};
use vleue_kinetoscope::{AnimatedImagePlugin, AnimationPlayed, StreamingAnimatedImageController};

use crate::engine::{
    asset_tracking::ResourceHandles,
    screens::{
        desktop::{DesktopAssets, DesktopTextures},
        Screen,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AnimatedImagePlugin);
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

#[derive(Component)]
struct StartupAnimationFinished(bool);

fn spawn_startup(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    window: Single<Entity, With<Window>>,
) {
    cmd.entity(*window)
        .insert((CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle: assets.load("ui/cursors/cursor_loading.png"),
            ..default()
        })),));
    cmd.spawn((
        StreamingAnimatedImageController::play(assets.load("ui/startup.gif")),
        DespawnOnEnter(Screen::Desktop),
        StartupAnimationFinished(false),
    ))
    .observe(
        |_: On<AnimationPlayed>, mut startup_anim: Single<&mut StartupAnimationFinished>| {
            startup_anim.0 = true;
        },
    );
}

// checks if all assets are loaded and if the startup gif has finished playing
fn is_startup_done(
    resource_handles: Res<ResourceHandles>,
    startup_anim: Query<&StartupAnimationFinished>,
) -> bool {
    let anim_finished = startup_anim.single().map(|f| f.0).unwrap_or(false);
    resource_handles.is_all_done() && anim_finished
}

fn enter_desktop_screen(
    mut next_screen: ResMut<NextState<Screen>>,
    mut cmd: Commands,
    window: Single<Entity, With<Window>>,
    d_ass: Res<DesktopAssets>,
) {
    info!("Entering desktop screen");
    next_screen.set(Screen::Desktop);
    cmd.entity(*window)
        .insert((CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle: d_ass.main_cursor.clone(),
            ..default()
        })),));
}
