use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::{
    engine::{minigames::ActiveMinigame, screens::Screen},
    ui::menus::desktop::TaskBarState,
};

pub mod asset_tracking;
pub mod audio;
pub mod design_scale;
pub mod file_system;
pub mod minigames;
pub mod screens;
pub mod scripted_events;
pub mod system_apps;
pub mod terminal_commands;
pub mod window_manager;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Pause>();
    app.configure_sets(
        Update,
        (CoreSystems::TickTimers, CoreSystems::Logic)
            .chain()
            .run_if(in_state(Pause(false))),
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
        minigames::plugin,
    ));
    app.add_systems(Update, pause_game.run_if(in_state(Screen::Desktop)));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Pause(pub bool);

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

fn pause_game(
    settings: Res<TaskBarState>,
    minigame: Res<ActiveMinigame>,
    current: Res<State<Pause>>,
    mut next: ResMut<NextState<Pause>>,
) {
    let should_pause = settings.settings_open || minigame.0.is_some();
    if current.get().0 != should_pause {
        next.set(Pause(should_pause));
    }
}
