use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use tracing::Level;

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
    .add_systems(Startup, setup_camera)
    .run();
}

// Initial Window
fn create_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Project Thallo".to_string(),
            resizable: false,
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
    commands.spawn((Name::new("Camera"), main_camera, projection));
}
