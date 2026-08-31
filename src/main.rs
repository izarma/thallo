use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{CursorOptions, WindowMode},
};
use bevy_egui::{EguiGlobalSettings, EguiPlugin};
use tracing::Level;

use crate::engine::{cursor::CurrentCursor, post_processing::PostProcessSettings};

#[cfg(feature = "dev")]
mod devtools;
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
    .add_systems(Startup, (setup_camera, spawn_cursor, setup_egui_input));

    #[cfg(feature = "dev")]
    app.add_plugins(devtools::plugin);

    app.run();
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
    let main_camera = Camera2d;
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
    mut current: ResMut<CurrentCursor>,
) {
    // disables egui messing with cursor
    egui_global.enable_cursor_icon_updates = false;
    // hide the OS cursor; we render the cursor ourselves so post-processing applies
    cmd.entity(*window).insert(CursorOptions {
        visible: false,
        ..default()
    });

    current.handle = assets.load("ui/cursors/cursor.png");
}

fn setup_egui_input(mut contexts: bevy_egui::EguiContexts) {
    if let Ok(ctx) = contexts.ctx_mut() {
        ctx.options_mut(|opt| opt.input_options.max_double_click_delay = 0.5);
    }
}
