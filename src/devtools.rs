use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::engine::{
    screens::Screen,
    scripted_events::{ScriptedEventTrigger, StoryBeat, UnlockState},
};
use crate::game::beats::ApplyBeatCommand;
use crate::ui::menus::Menu;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
    );

    app.add_systems(
        Update,
        (story_skip_hotkeys, game_over_hotkeys, log_unlock_state),
    );
    app.add_systems(EguiPrimaryContextPass, beat_jump_panel);
}

/// Play-test shortcuts for jumping between story beats.
///
/// - `F5` -> rebuild the Act 1 connecting desktop.
/// - `F6` -> rebuild the Act 2 desktop.
fn story_skip_hotkeys(
    mut cmd: Commands,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::F5) {
        info!("[Dev] Skip -> Act 1 connecting");
        jump_to_beat(
            StoryBeat::Act1Connecting,
            &mut cmd,
            &mut next_screen,
            &mut next_menu,
        );
    }
    if keys.just_pressed(KeyCode::F6) {
        info!("[Dev] Skip -> Act 2");
        jump_to_beat(
            StoryBeat::Act2Sos,
            &mut cmd,
            &mut next_screen,
            &mut next_menu,
        );
    }
}

/// Play-test shortcuts for jumping straight to the game-over screen.
///
/// - `F9` -> instantly win (show the ending).
/// - `F10` -> instantly lose (show the death screen).
fn game_over_hotkeys(mut cmd: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F9) {
        info!("[Dev] Skip -> Win");
        cmd.trigger(ScriptedEventTrigger::Win);
    }

    if keys.just_pressed(KeyCode::F10) {
        info!("[Dev] Skip -> Lose");
        cmd.trigger(ScriptedEventTrigger::RipperFailed);
    }
}

/// Print the current [`UnlockState`] to the log when `Insert` is pressed.
fn log_unlock_state(state: Res<UnlockState>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Insert) {
        info!("[Dev] Current UnlockState: {:?}", *state);
    }
}

/// egui dev panel (next to the world inspector) listing every [`StoryBeat`].
fn beat_jump_panel(
    mut contexts: EguiContexts,
    mut cmd: Commands,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    state: Res<UnlockState>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    bevy_egui::egui::Window::new("Story Beats").show(ctx, |ui| {
        ui.label(format!("Current: {:?}", state.story.beat));
        for beat in StoryBeat::ALL {
            if ui.button(format!("{beat:?}")).clicked() {
                jump_to_beat(beat, &mut cmd, &mut next_screen, &mut next_menu);
            }
        }
    });
    Ok(())
}

/// Route a beat jump to the right `Screen`/`Menu`, rebuilding state via
/// [`apply_beat`](crate::game::beats::apply_beat).
fn jump_to_beat(
    beat: StoryBeat,
    cmd: &mut Commands,
    next_screen: &mut NextState<Screen>,
    next_menu: &mut NextState<Menu>,
) {
    match beat {
        StoryBeat::Win => cmd.trigger(ScriptedEventTrigger::Win),
        StoryBeat::Lose => cmd.trigger(ScriptedEventTrigger::RipperFailed),
        other => {
            cmd.queue(ApplyBeatCommand(other));
            next_menu.set(Menu::None);
            next_screen.set(Screen::Desktop);
        }
    }
}
