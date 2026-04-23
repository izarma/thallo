use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::engine::{
    CoreSystems,
    screens::Screen,
    system_apps::{Applications, ChatBoxState, OpenAppEvent},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UnlockState>();
    //.load_resource::<MinigameAssets>();
    app.init_resource::<FileDialogueTriggers>();
    app.add_observer(process_scripted_event);
    app.add_observer(on_file_opened);
    app.add_systems(OnEnter(Screen::Desktop), schedule_open_chat);
    app.add_systems(
        Update,
        tick_delayed_events
            .in_set(CoreSystems::Logic)
            .run_if(in_state(Screen::Desktop)),
    );

    app.init_resource::<FileDialogueTriggers>();
    // app.init_resource::<ActiveMinigame>()
    //     .add_observer(on_minigame_trigger);
}

#[derive(Event, Clone)]
pub enum ScriptedEvent {
    OpenChat,
    Unlock1,
    FileOpened(String),
}

#[derive(Component)]
struct DelayedEvent {
    timer: Timer,
    event: ScriptedEvent,
}

#[derive(Resource, Default, Debug)]
pub struct UnlockState {
    pub chat: bool,
    pub mg1: bool,
}

fn process_scripted_event(
    ev: On<ScriptedEvent>,
    mut state: ResMut<UnlockState>,
    mut cmd: Commands,
    mut triggers: ResMut<FileDialogueTriggers>,
    mut dialogues: ResMut<Dialogues>,
) {
    match ev.clone() {
        ScriptedEvent::OpenChat => {
            if !state.chat {
                state.chat = true;
                info!("[Story] Chat unlocked");
                cmd.trigger(OpenAppEvent {
                    name: "Chat".to_string(),
                    app_type: Applications::Chatbox {
                        input: String::new(),
                        state: ChatBoxState::AnonTyping { elapsed: 0.0 },
                        displayed: Vec::new(),
                    },
                });
            }
        }
        ScriptedEvent::Unlock1 => {
            if !state.mg1 {
                state.mg1 = true;
                info!("[Story] omega.png decrypted — Unlock1 complete");
            }
        }
        ScriptedEvent::FileOpened(name) => {
            if triggers.triggered.contains(name.as_str()) {
                return;
            }
            if let Some(lines) = triggers.map.get(name.as_str()).cloned() {
                info!("[Story] File '{}' dialogue triggered", name);
                triggers.triggered.insert(name);
                dialogues.add_lines(lines);

                // Re-open or un-minimize the chatbox.
                // If it was closed, this creates a fresh instance starting at the new lines.
                // If it was minimized, this brings it back up.
                cmd.trigger(OpenAppEvent {
                    name: "Chat".to_string(),
                    app_type: Applications::Chatbox {
                        input: String::new(),
                        state: ChatBoxState::AnonTyping { elapsed: 0.0 },
                        displayed: Vec::new(),
                    },
                });
            }
        }
    }
}

fn schedule_open_chat(mut cmd: Commands) {
    cmd.spawn(DelayedEvent {
        timer: Timer::from_seconds(5.0, TimerMode::Once),
        event: ScriptedEvent::OpenChat,
    });
}

fn tick_delayed_events(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut DelayedEvent)>,
    time: Res<Time>,
) {
    for (entity, mut delayed) in &mut query {
        delayed.timer.tick(time.delta());
        if delayed.timer.just_finished() {
            cmd.trigger(delayed.event.clone());
            cmd.entity(entity).despawn();
        }
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
    pub fn has_unplayed(&self) -> bool {
        self.index < self.lines.len()
    }
}

/// `speaker: false` = anon, `speaker: true` = player.
#[derive(Debug, PartialEq, Clone)]
pub struct DialogueLine {
    pub speaker: bool,
    pub text: String,
}

impl DialogueLine {
    pub fn new(speaker: bool, text: &str) -> Self {
        Self {
            speaker,
            text: text.to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub struct FileDialogueTriggers {
    pub map: HashMap<String, Vec<DialogueLine>>,
    triggered: HashSet<String>,
}

impl FileDialogueTriggers {
    pub fn register(&mut self, file_name: impl Into<String>, lines: Vec<DialogueLine>) {
        self.map.insert(file_name.into(), lines);
    }
}

fn on_file_opened(ev: On<OpenAppEvent>, mut cmd: Commands) {
    let is_file_app = matches!(
        ev.app_type,
        Applications::TextViewer { .. } | Applications::ImageViewer { .. }
    );
    if !is_file_app {
        return;
    }
    cmd.spawn(DelayedEvent {
        timer: Timer::from_seconds(5.0, TimerMode::Once),
        event: ScriptedEvent::FileOpened(ev.name.clone()),
    });
}

fn next_chatbox_state(lines: &[DialogueLine], index: usize) -> ChatBoxState {
    match lines.get(index) {
        Some(line) if !line.speaker => ChatBoxState::AnonTyping { elapsed: 0.0 },
        Some(_) => ChatBoxState::PlayerReady { chars_revealed: 0 },
        None => ChatBoxState::Done,
    }
}
