use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::design_scale::DesignScale,
    ui::{menus::Menu, theme::widgets::primitives},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::ShutDown), spawn_shutdown_timer);
    app.add_systems(Update, tick_shutdown_timer.run_if(in_state(Menu::ShutDown)));
    app.add_systems(
        EguiPrimaryContextPass,
        shutdown_ui.run_if(in_state(Menu::ShutDown)),
    );
}

fn spawn_shutdown_timer(mut cmd: Commands) {
    cmd.insert_resource(ShutdownTimer(Timer::from_seconds(5.0, TimerMode::Once)));
}

/// Draws the shutdown screen with egui every frame while in [`Menu::ShutDown`].
fn shutdown_ui(
    mut contexts: EguiContexts,
    timer: Res<ShutdownTimer>,
    scale: Res<DesignScale>,
) -> Result {
    let remaining = (timer.0.remaining_secs().ceil() as u32).max(0);
    let ctx = contexts.ctx_mut()?;

    primitives::centered_panel(ctx, "shutdown_menu", |ui| {
        primitives::header(ui, "ACCESS DENIED, UNIDENTIFIED ENTRY", &scale);
        ui.add_space(8.0);
        primitives::header(ui, format!("{remaining}"), &scale);
    });

    Ok(())
}

#[derive(Resource)]
struct ShutdownTimer(Timer);

fn tick_shutdown_timer(
    mut timer: ResMut<ShutdownTimer>,
    time: Res<Time>,
    mut app_exit: MessageWriter<AppExit>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        app_exit.write(AppExit::Success);
    }
}
