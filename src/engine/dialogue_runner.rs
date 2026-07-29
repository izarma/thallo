use bevy::prelude::*;

use crate::engine::{
    CoreSystems,
    audio::sound_effect,
    screens::{Screen, desktop::DesktopAssets},
    scripted_events::{ScriptedEventTrigger, UnlockState},
    system_apps::ChatBoxState,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DialogueRunner>();
    app.add_systems(
        Update,
        tick_dialogue_runner
            .in_set(CoreSystems::Logic)
            .run_if(in_state(Screen::Desktop)),
    );
}

/// The single source of truth for the chatbox dialogue state machine.
/// Lives independent of the chatbox window so it ticks even when minimised.
#[derive(Resource)]
pub struct DialogueRunner {
    /// Current state of the conversation turn.
    pub state: ChatBoxState,
    /// Lines that have already been committed to the chat log.
    pub displayed: Vec<DialogueLine>,
    /// The player's current (partially revealed) typed response.
    pub input: String,
}

impl Default for DialogueRunner {
    fn default() -> Self {
        Self {
            state: ChatBoxState::Done,
            displayed: Vec::new(),
            input: String::new(),
        }
    }
}

/// `speaker: false` = anon, `speaker: true` = player.
#[derive(Debug, PartialEq, Clone)]
pub struct DialogueLine {
    pub speaker: bool,
    pub text: String,
    pub on_complete: Option<ScriptedEventTrigger>,
}

impl DialogueLine {
    pub fn new(speaker: bool, text: &str) -> Self {
        Self {
            speaker,
            text: text.to_string(),
            on_complete: None,
        }
    }
    pub fn on_complete(mut self, event: ScriptedEventTrigger) -> Self {
        self.on_complete = Some(event);
        self
    }
}

#[derive(Resource)]
pub struct Dialogues {
    pub lines: Vec<DialogueLine>,
    pub index: usize,
}

impl Dialogues {
    pub fn add_lines(&mut self, lines: Vec<DialogueLine>) {
        self.lines.extend(lines);
    }
}

/// Derive the correct ChatBoxState for the line now at `index`.
/// Used both by the tick system and by callers that resume the runner.
pub fn derive_state(lines: &[DialogueLine], index: usize) -> ChatBoxState {
    match lines.get(index) {
        Some(line) if !line.speaker => ChatBoxState::AnonTyping { elapsed: 0.0 },
        Some(_) => ChatBoxState::PlayerReady { chars_revealed: 0 },
        None => ChatBoxState::Done,
    }
}

/// Chars/second at which anon "types" his messages.
pub const TYPING_SPEED: f32 = 18.0;

fn tick_dialogue_runner(
    mut runner: ResMut<DialogueRunner>,
    mut dialogues: ResMut<Dialogues>,
    time: Res<Time>,
    mut cmd: Commands,
    assets: Option<Res<DesktopAssets>>,
    state: Res<UnlockState>,
) {
    if !state.programs.chat {
        return;
    }

    // Auto-resume after new lines are appended while runner was Done.
    if matches!(runner.state, ChatBoxState::Done) {
        if dialogues.lines.get(dialogues.index).is_some() {
            runner.state = derive_state(&dialogues.lines, dialogues.index);
            if matches!(runner.state, ChatBoxState::PlayerReady { .. }) {
                play_notification_sfx(assets.as_deref(), &mut cmd);
            }
        } else {
            return;
        }
    }

    // AnonTyping tick
    let should_advance = if let ChatBoxState::AnonTyping { elapsed } = &mut runner.state {
        let Some(line) = dialogues.lines.get(dialogues.index) else {
            runner.state = ChatBoxState::Done;
            return;
        };
        *elapsed += time.delta_secs();
        (*elapsed * TYPING_SPEED) as usize >= line.text.chars().count()
    } else {
        false
    };

    if should_advance {
        if let Some(line) = dialogues.lines.get(dialogues.index) {
            let text = line.text.clone();
            runner.displayed.push(DialogueLine::new(false, &text));
        }
        dialogues.index += 1;
        runner.state = derive_state(&dialogues.lines, dialogues.index);
        runner.input.clear();
        play_notification_sfx(assets.as_deref(), &mut cmd);
    }
}

fn play_notification_sfx(assets: Option<&DesktopAssets>, cmd: &mut Commands) {
    if let Some(assets) = assets {
        cmd.spawn(sound_effect(assets.msg_notification.clone()));
    }
}
