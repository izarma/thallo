use bevy::{
    prelude::*,
    window::{CursorIcon, CustomCursor, CustomCursorImage},
};
use bevy_egui::EguiGlobalSettings;

mod apps;
pub mod menus;
mod theme;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((theme::plugin, menus::plugin, apps::plugin));
    app.add_systems(Startup, spawn_cursor);
}

fn spawn_cursor(
    mut cmd: Commands,
    window: Single<Entity, With<Window>>,
    assets: Res<AssetServer>,
    mut egui_global: ResMut<EguiGlobalSettings>,
) {
    //disables egui messing with cursor
    egui_global.enable_cursor_icon_updates = false;
    cmd.entity(*window)
        .insert((CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle: assets.load("ui/cursors/cursor.png"),
            ..default()
        })),));
}
