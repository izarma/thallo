use bevy::prelude::*;
use bevy_egui::{EguiTextureHandle, EguiUserTextures, egui};

use crate::engine::{
    Pause,
    dialogue_runner::{DialogueRunner, Dialogues},
    minigames::ActiveMinigame,
    scripted_events::{ACT2_TIME_LIMIT, Act2Timer, FileDialogueTriggers, StoryBeat, UnlockState},
    system_apps::ChatBoxState,
    window_manager::OpenWindows,
};
use crate::game::{
    FileAssets,
    dialogues::{
        build_act1_dialogues, build_act1_file_triggers, build_act2_dialogues,
        build_netconn_file_triggers,
    },
    files::build_fs_for_beat,
};
use crate::ui::popups::OpenAlerts;

/// Rebuild every piece of state that depends on narrative position for `beat`.
///
/// This is the single reset/rebuild path: game events, Retry and dev jumps all
/// land here. It is idempotent and safe to call from anywhere with `&mut World`
/// (typically via [`ApplyBeatCommand`]).
pub fn apply_beat(world: &mut World, beat: StoryBeat) {
    // 1. Story flags + program unlocks.
    {
        let mut state = world.resource_mut::<UnlockState>();
        state.apply_beat(beat);
    }

    // 2. Filesystem, rebuilt from scratch for the beat.
    let (omega_tex, size) = resolve_omega(world);
    world.insert_resource(build_fs_for_beat(beat, omega_tex, size));

    // 3. Dialogues + runner.
    world.insert_resource(dialogues_for_beat(beat));
    {
        let mut runner = world.resource_mut::<DialogueRunner>();
        runner.displayed.clear();
        runner.input.clear();
        runner.state = ChatBoxState::Done;
    }

    // 4. File dialogue triggers (rebuilt *and* `opened` cleared).
    {
        let mut triggers = world.resource_mut::<FileDialogueTriggers>();
        triggers.clear();
        triggers_for_beat(beat, &mut triggers);
    }

    // 5. Act 2 countdown: active exactly for the SOS beat, reset to full.
    {
        let mut timer = world.resource_mut::<Act2Timer>();
        timer.active = matches!(beat, StoryBeat::Act2Sos);
        timer.remaining = ACT2_TIME_LIMIT;
    }

    // 6. Transient UI / game state.
    world.resource_mut::<OpenWindows>().windows.clear();
    world.resource_mut::<OpenAlerts>().clear();
    world.resource_mut::<ActiveMinigame>().0 = None;
    world.resource_mut::<NextState<Pause>>().set(Pause(false));
}

/// A [`Command`] wrapper so systems/observers can queue [`apply_beat`] without
/// needing `&mut World` in their own signature.
pub struct ApplyBeatCommand(pub StoryBeat);

impl Command for ApplyBeatCommand {
    fn apply(self, world: &mut World) {
        apply_beat(world, self.0);
    }
}

fn dialogues_for_beat(beat: StoryBeat) -> Dialogues {
    match beat {
        StoryBeat::Act1Intro => build_act1_dialogues(),
        StoryBeat::Act2Sos => build_act2_dialogues(),
        StoryBeat::Act1Connecting | StoryBeat::Win | StoryBeat::Lose => Dialogues {
            lines: Vec::new(),
            index: 0,
        },
    }
}

fn triggers_for_beat(beat: StoryBeat, triggers: &mut FileDialogueTriggers) {
    match beat {
        StoryBeat::Act1Intro => build_act1_file_triggers(triggers),
        StoryBeat::Act1Connecting => build_netconn_file_triggers(triggers),
        StoryBeat::Act2Sos | StoryBeat::Win | StoryBeat::Lose => {}
    }
}

/// Resolve the `omega` texture id + size, or fall back to a default size when
/// `FileAssets` has not finished loading yet (e.g. an early dev jump).
fn resolve_omega(world: &mut World) -> (Option<egui::TextureId>, egui::Vec2) {
    let Some(omega_handle) = world
        .get_resource::<FileAssets>()
        .map(|assets| assets.omega.clone())
    else {
        return (None, egui::Vec2::splat(64.0));
    };

    let size = world
        .get_resource::<Assets<Image>>()
        .and_then(|images| images.get(&omega_handle))
        .map(|img| {
            let s = img.size_f32();
            egui::Vec2::new(s.x, s.y)
        })
        .unwrap_or(egui::Vec2::splat(64.0));

    let tex = world
        .resource_mut::<EguiUserTextures>()
        .add_image(EguiTextureHandle::Weak(omega_handle.id()));

    (Some(tex), size)
}
