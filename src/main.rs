use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{CursorIcon, CustomCursor, CustomCursorImage, WindowMode},
};
use bevy_egui::{EguiGlobalSettings, EguiPlugin};
use tracing::Level;

use crate::engine::post_processing::PostProcessSettings;

mod engine;
mod game;
mod ui;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(create_window_plugin()).set(LogPlugin {
            level: Level::DEBUG,
            ..default()
        }),
        EguiPlugin::default(),
        engine::plugin,
        ui::plugin,
        game::plugin,
    ))
    // this turns into the default background color
    .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
    .add_systems(Startup, (setup_camera, spawn_cursor))
    .run();
}

// Initial Window
fn create_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Project Thallo".to_string(),
            resizable: false,
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            ..default()
        }),
        ..default()
    }
}

fn setup_camera(mut commands: Commands) {
    let main_camera = Camera2d::default();
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::Fixed {
            width: (1280.0),
            height: (720.0),
        },
        ..OrthographicProjection::default_2d()
    });
    commands.spawn((
        Name::new("Camera"),
        main_camera,
        projection,
        PostProcessSettings {
            intensity: 0.02,
            ..default()
        },
    ));
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
