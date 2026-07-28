use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::engine::scripted_events::{RebootSequence, ScriptedEventTrigger, UnlockState};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
    );

    app.add_systems(
        Update,
        (
            story_skip_hotkeys,
            unlock_state_toggle_hotkeys,
            log_unlock_state,
        ),
    );
}

/// Play-test shortcuts for jumping between story beats.
///
/// - `F5` -> trigger the Act 1 break (`NetworkConnect` reboot).
/// - `F6` -> trigger the Act 2 transition (`ActTrans` reboot).
fn story_skip_hotkeys(mut cmd: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F5) {
        info!("[Dev] Skip -> Act 1 break");
        cmd.trigger(ScriptedEventTrigger::BeginReboot(
            RebootSequence::NetworkConnect,
        ));
    }

    if keys.just_pressed(KeyCode::F6) {
        info!("[Dev] Skip -> Act 2");
        cmd.trigger(ScriptedEventTrigger::BeginReboot(RebootSequence::ActTrans));
    }
}

/// Toggle individual flags in [`UnlockState`] for manual play-test control.
///
/// - `F7`  -> bruteforce
/// - `F8`  -> netripper
fn unlock_state_toggle_hotkeys(mut state: ResMut<UnlockState>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F7) {
        state.bruteforce = !state.bruteforce;
        info!("[Dev] UnlockState.bruteforce = {}", state.bruteforce);
    }
    if keys.just_pressed(KeyCode::F8) {
        state.netripper = !state.netripper;
        info!("[Dev] UnlockState.netripper = {}", state.netripper);
    }
}

/// Print the current [`UnlockState`] to the log when `Insert` is pressed.
fn log_unlock_state(state: Res<UnlockState>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Insert) {
        info!("[Dev] Current UnlockState: {:?}", *state);
    }
}
