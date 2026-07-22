use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::{
    engine::{
        audio::sound_effect, design_scale::DesignScale, screens::Screen,
        scripted_events::UnlockState,
    },
    game::FileAssets,
    ui::{menus::Menu, theme::widgets::primitives},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::ActBreak),
        (spawn_break_timer, spawn_actbreak_sfx),
    );
    app.add_systems(Update, tick_break_timer.run_if(in_state(Screen::ActBreak)));
    app.add_systems(
        EguiPrimaryContextPass,
        (
            connecting_sunday.run_if(in_state(Menu::ConnectingSunday)),
            act1_break_ui.run_if(in_state(Menu::Act1Break)),
        ),
    );
}

fn connecting_sunday(mut contexts: EguiContexts, scale: Res<DesignScale>) -> Result {
    let ctx = contexts.ctx_mut()?;
    primitives::centered_panel(ctx, "connecting_sunday", |ui| {
        primitives::header(ui, "Connecting to Sunday...", &scale);
    });
    Ok(())
}

fn act1_break_ui(mut contexts: EguiContexts, scale: Res<DesignScale>) -> Result {
    let ctx = contexts.ctx_mut()?;
    primitives::centered_panel(ctx, "act2", |ui| {
        primitives::header(ui, "The Message", &scale);
    });
    Ok(())
}

fn spawn_break_timer(mut cmd: Commands) {
    cmd.insert_resource(BreakTimer(Timer::from_seconds(10.0, TimerMode::Once)));
}

fn spawn_actbreak_sfx(mut cmd: Commands, assets: Res<FileAssets>, state: Res<UnlockState>) {
    if state.network_reconnected && !state.act {
        info!("break sfx playing");
        cmd.spawn(sound_effect(assets.tans_act.clone()));
    } else if state.act {
        cmd.spawn(sound_effect(assets.tans_act.clone()));
    }
}

#[derive(Resource)]
struct BreakTimer(Timer);

fn tick_break_timer(
    mut timer: ResMut<BreakTimer>,
    time: Res<Time>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        next_screen.set(Screen::Title);
    }
}
