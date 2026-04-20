use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub mod asset_tracking;
pub mod audio;
pub mod design_scale;
pub mod file_system;
pub mod screens;
pub mod scripted_events;
pub mod system_apps;
pub mod terminal_commands;
pub mod window_manager;

pub(super) fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (CoreSystems::TickTimers, CoreSystems::Logic).chain(),
    )
    .configure_sets(
        EguiPrimaryContextPass,
        (UiPassSystems::CacheTextures, UiPassSystems::Render).chain(),
    );
    app.add_plugins((
        asset_tracking::plugin,
        audio::plugin,
        screens::plugin,
        design_scale::plugin,
        window_manager::plugin,
        scripted_events::plugin,
    ));
}

/// Core Systemset for Simulation logic
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CoreSystems {
    /// Update: tick timers before anything reads their output
    TickTimers,
    /// Update: game logic, VFS mutations, state writes
    Logic,
}

/// EguiPrimaryContextPass Systemset
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UiPassSystems {
    /// EguiPrimaryContextPass: cache Bevy handles → egui TextureIds
    CacheTextures,
    /// EguiPrimaryContextPass: all rendering (desktop, windows, taskbar)
    Render,
}
